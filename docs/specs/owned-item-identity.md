# Owned-item identity

A bag item and an equipped item can carry a supplied opaque identity; an item moved to the cursor carries the same identity. Bag pickup, bag placement, ordinary equipment pickup/equip and backpack placement retain it. A split-off stack is a distinct item and does not inherit the source identity. Replacing an item with a newly supplied item does not inherit the replaced identity.

The identity is simulator state, not a native GUID or a persistent server identity. The current item snapshot contains no owned-item GUIDs. Server equip results and identities across session or snapshot reload are outside this contract.
