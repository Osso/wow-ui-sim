# New transmog appearance

Retail 12.1.5 declares `C_TransmogCollection.IsNewAppearance(visualID)` with one required number and one boolean result in `Blizzard_APIDocumentationGenerated/TransmogItemsDocumentation.lua:730–742`.

The result is true when a source associated with the visual belongs to the supplied new-source set. It is false for an absent group or no matching source. The query accepts numeric-convertible signed 32-bit inputs, truncating fractions, and rejects inaccessible secrets and invalid inputs. The original wardrobe uses the result to show or hide a new badge (`Blizzard_Collections/Mainline/Blizzard_Wardrobe.lua:1126–1132`).

**EMULATED:** The host supplies new-source membership; the initial set is empty. Best-guess basis: Retail 12.1.5 build 70688 local predicate and source-set evidence. **Known gaps:** acquisition, persistence, source removal, `ClearNewAppearance`, and associated notifications are not modeled by this query.
