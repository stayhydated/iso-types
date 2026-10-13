"""Select current, common ISO data from pinned CLDR metadata."""
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET


def expand_range(token):
    """Expand CLDR's abbreviated, fixed-width alphanumeric ranges."""
    if '~' not in token:
        return {token}
    start, suffix = token.split('~')
    end = start[:-len(suffix)] + suffix
    assert len(start) == len(end)
    result = set()
    current = start
    while current <= end:
        result.add(current)
        if current == end:
            return result
        chars = list(current)
        for index in range(len(chars) - 1, -1, -1):
            alphabet = ('0123456789' if chars[index].isdigit() else
                        'abcdefghijklmnopqrstuvwxyz' if chars[index].islower() else
                        'ABCDEFGHIJKLMNOPQRSTUVWXYZ')
            next_index = alphabet.index(chars[index]) + 1
            if next_index < len(alphabet):
                chars[index] = alphabet[next_index]
                break
            chars[index] = alphabet[0]
        current = ''.join(chars)
    raise ValueError(f'invalid CLDR range: {token}')


class CurrentData:
    def __init__(self, data):
        self.path = Path(data) / 'current'
        self.provenance = json.loads((self.path / 'provenance.json').read_text())
        for name, source in self.provenance['files'].items():
            if hashlib.sha256((self.path / name).read_bytes()).hexdigest() != source['sha256']:
                raise ValueError(f'current-data source checksum mismatch: {name}')
        self.as_of = self.provenance['as_of']
        metadata = ET.parse(self.path / 'supplementalMetadata.xml')
        self.language_aliases = {r.get('type'): (r.get('replacement'), r.get('reason'))
                                 for r in metadata.findall('.//languageAlias')}
        self.region_aliases = {r.get('type'): (r.get('replacement'), r.get('reason'))
                               for r in metadata.findall('.//territoryAlias')}
        supplemental = ET.parse(self.path / 'supplementalData.xml')
        self.modern_locales = set()
        for line in (self.path / 'coverageLevels.txt').read_text().splitlines():
            line = line.split('#')[0].strip()
            if line:
                locale, level, *_ = [part.strip() for part in line.split(';')]
                if level == 'modern':
                    self.modern_locales.add(locale.replace('_', '-'))
        # Explicit CLDR parents supplement ordinary subtag truncation.
        parents = supplemental.findall('.//parentLocales/parentLocale')
        changed = True
        while changed:
            changed = False
            for parent in parents:
                if parent.get('parent').replace('_', '-') in self.modern_locales:
                    for child in parent.get('locales').split():
                        child = child.replace('_', '-')
                        if child not in self.modern_locales:
                            self.modern_locales.add(child)
                            changed = True
        self.languages = {locale.split('-')[0] for locale in self.modern_locales}
        self.scripts = set()
        for line in (self.path / 'scriptMetadata.txt').read_text().splitlines():
            line = line.split('#')[0].strip()
            if line:
                fields = [part.strip() for part in line.split(';')]
                if fields[5] == 'RECOMMENDED':
                    self.scripts.add(fields[0])
        self.scripts &= self.validity('script', {'regular'})
        likely = {row.get('from'): row.get('to') for row in
                  ET.parse(self.path / 'likelySubtags.xml').findall('.//likelySubtag')}
        self.languages = {language for language in self.languages
                          if likely.get(language, '').split('_')[1:2]
                          and likely[language].split('_')[1] in self.scripts}
        self.regions = self.validity('region', {'regular', 'macroregion'})
        self.subdivisions = self.validity('subdivision', {'regular'})
        # Tender currencies whose circulation interval includes the snapshot date.
        self.currencies = {row.get('iso4217') for row in supplemental.findall('.//currencyData/region/currency')
                           if row.get('tender') != 'false'
                           and row.get('from', '0000-01-01') <= self.as_of
                           and row.get('to', '9999-12-31') >= self.as_of}
        current_iso = json.loads((self.path / 'current_currencies.json').read_text())
        self.currencies &= set(current_iso['codes'])
        self.language_inputs = set(self.languages)
        for code, (_, reason) in self.language_aliases.items():
            if reason in {'overlong', 'macrolanguage'} and self.language(code) in self.languages:
                self.language_inputs.add(code)
        self.region_inputs = set(self.regions)
        for code, (replacement, reason) in self.region_aliases.items():
            if reason == 'overlong' and replacement in self.regions:
                self.region_inputs.add(code)
        self.living_languages = {row['alpha_3'] for row in json.loads((Path(data) / 'iso_639-3.json').read_text())['639-3']
                                 if row['type'] == 'L'}

    def validity(self, kind, statuses):
        return {code for row in ET.parse(self.path / f'{kind}.xml').findall('.//id')
                if row.get('idStatus') in statuses
                for token in (row.text or '').split() for code in expand_range(token)}

    def language(self, code):
        seen = set()
        while code in self.language_aliases:
            if code in seen:
                raise ValueError(f'cyclic CLDR language alias: {code}')
            seen.add(code)
            replacement, reason = self.language_aliases[code]
            if reason not in {'overlong', 'macrolanguage'}:
                return None
            code = replacement.split('_')[0]
        return code

    def includes(self, standard, record, code_key):
        code = record[code_key]
        if standard in {'639-1', '639-2', '639-3'}:
            alpha_3 = record['alpha_3']
            return alpha_3 in self.living_languages and self.language(code) in self.languages
        if standard == '15924':
            return code in self.scripts
        if standard == '4217':
            return code in self.currencies
        if standard == '3166-1':
            return code in self.regions
        if standard == '3166-2':
            return code.replace('-', '').lower() in self.subdivisions
        return True

    def rust_locale_policy(self):
        def table(name, values):
            return f'const {name}: &[&str] = &[' + ', '.join(json.dumps(v) for v in sorted(values)) + '];\n'
        output = '// Generated by scripts/generate.py from pinned current-data policy.\n'
        output += table('LANGUAGES', self.language_inputs)
        output += table('SCRIPTS', self.scripts)
        output += table('REGIONS', self.region_inputs)
        output += '''pub(crate) fn current_locale(locale: &icu_locale::LanguageIdentifier) -> bool {
    LANGUAGES.binary_search(&locale.language.as_str()).is_ok()
        && locale.script.is_none_or(|script| SCRIPTS.binary_search(&script.as_str()).is_ok())
        && locale.region.is_none_or(|region| REGIONS.binary_search(&region.as_str()).is_ok())
        && locale.variants.is_empty()
}
'''
        return output
