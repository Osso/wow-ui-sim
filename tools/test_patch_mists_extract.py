"""Mists source wrappers retain links, not markup or expanded content."""
import unittest
from extract_patch_non_inventory import extract_text


class MistsExtractTests(unittest.TestCase):
    def test_external_link_and_noinclude_wrapper(self):
        raw = ('<noinclude>{{Transclude|}}</noinclude>\n==External links==\n'
               '{{Elink|site=WoWInterface|link=http://example.test/thread|desc=Assorted changes}}\n')
        self.assertEqual(extract_text(raw, mists_source_markup=True),
                         '\n== External links ==\n[External link: WoWInterface; http://example.test/thread; Assorted changes]\n')


if __name__ == '__main__':
    unittest.main()
