# Forever Edit Mode default layout

For `client-wowforever`, `C_EditMode.GetEditModeDefaultLayout()` returns the named Modern preset (`0`). This is the initialized simulator profile default, grounded in the user-supplied build 1.60.1.69913 screenshot and the existing preset enum.

Saved layouts and active-layout selection are separate state. Input-interface style changes do not alter this default. Native dynamic-selection rules remain unknown; this contract does not claim them. Other profiles retain their existing API surface.

The unchanged Camelot preset constants, Mainline preset layouts, and shared preset manager must resolve default system anchors using this value and return independent copies. No vendor behavior is patched.
