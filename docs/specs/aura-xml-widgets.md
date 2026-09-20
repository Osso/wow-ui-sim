# Aura XML Widgets

## Contract

- Retail 12.1+ and Forever support `AuraContainer`, `ManagedAuraContainer`, and `AuraButton` in root and nested XML elements.
- Aura container aliases use the existing Frame implementation; AuraButton uses Button. Their intrinsic template names remain available to the XML loader and `CreateFrame`.
- Template inheritance, mixins, parent keys, frame methods, and OnLoad/OnUpdate dispatch use the existing runtime paths, including forbidden-partition private scripts.
- Earlier profile bundles do not gain these aliases. A shared `aura-xml-widgets` capability selects the parser variants, frame-data conversion, and widget mappings without enabling a retail epoch for Forever.

## Evidence and tests

Forever 1.60.1.69913 `Blizzard_UnitFrame/Shared/TargetFrameAuraContainer.xml` uses a scoped `AuraContainer` template with private mixins and method-bound scripts. The previous Forever loader rejected nested aura elements as unknown variants.

`tests/wowforever_aura_xml.rs` exercises all three aliases through the addon/XML loader and factory, and the target template's scoped/private script structure with concrete fixture methods. The fixture covers frame parenting, dimensions, event registration, button text, OnLoad, and OnUpdate. It does not replace vendor code or claim full unit/aura behavior or native security conformance.

## Related

- [Client profiles](client-profiles.md)
- [Forever patch report](../wowforever-1.60.1.md)
