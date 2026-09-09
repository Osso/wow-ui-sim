-- Gethe 49b69918fcdc77e109813281e4f537d45ec7dcbf: publication only.
Enum.BreakType = { Character = 0, Word = 1, Sentence = 2, Line = 3 }
Enum.BreakTypeMeta = { MinValue = 0, MaxValue = 3, NumValues = 4 }

Enum.CollationStrength = { Primary = 0, Secondary = 1, Tertiary = 2, Quaternary = 3, Identical = 4 }
Enum.CollationStrengthMeta = { MinValue = 0, MaxValue = 4, NumValues = 5 }

Enum.CurrencyNameStyle = { Symbol = 0, NarrowSymbol = 1, Long = 2, FormalSymbol = 3, VariantSymbol = 4 }
Enum.CurrencyNameStyleMeta = { MinValue = 0, MaxValue = 4, NumValues = 5 }

Enum.DateTimeStyle = { None = 0, Short = 1, Medium = 2, Long = 3, Full = 4 }
Enum.DateTimeStyleMeta = { MinValue = 0, MaxValue = 4, NumValues = 5 }

Enum.LocaleTransform = {
  Canonicalize = 0, AddLikelySubtags = 1, RemoveLikelySubtags = 2, Language = 3,
  Script = 4, Region = 5, Variant = 6, ParentLocale = 7,
}
Enum.LocaleTransformMeta = { MinValue = 0, MaxValue = 7, NumValues = 8 }

Enum.NormalizationForm = { Nfc = 0, Nfd = 1, Nfkc = 2, Nfkd = 3 }
Enum.NormalizationFormMeta = { MinValue = 0, MaxValue = 3, NumValues = 4 }

Enum.NumberStyle = { Decimal = 0, Integer = 1, Percent = 2, Currency = 3 }
Enum.NumberStyleMeta = { MinValue = 0, MaxValue = 3, NumValues = 4 }

Enum.PluralType = { Cardinal = 0, Ordinal = 1 }
Enum.PluralTypeMeta = { MinValue = 0, MaxValue = 1, NumValues = 2 }
