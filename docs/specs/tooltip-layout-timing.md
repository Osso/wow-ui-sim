# Tooltip layout timing

For an owned tooltip with content, adding a line or double line and changing
custom word-wrap width, custom line spacing, shrink-to-fit, or minimum width
must not synchronously resize it. A subsequent `Show` sizes its current content.
`SetPadding` synchronously lays out the current content. Repeated `Show` also
performs layout. Native evidence: WM63-L4 and WM63-L23, Retail 12.1.5.70688.

These timing rules do not establish native font metrics, markup widths, or
money and status-bar heights. The simulator's font sizes and line-height
multiplier are estimates, not native measurements. GUI builds use glyph-backed
measurement at these triggers; builds without GUI use an estimator. Geometry
and padding placement are outside this timing slice. For Retail/PTR, outer width
and height add 20.48 plus their explicit sides after content sizing. Padding
sides are right, bottom, left, top; omitted left and top preserve their prior
values. This arithmetic follows WM63-L23 (Retail 12.1.5.70688). Render and periodic
update do not recalculate Retail/PTR content size; they resolve stored geometry.
WM63-L4 line 27 verifies that the native periodic-update override does not call
the content-sizing routine. WM63 is evidence for
Retail 70688, not Wrath, Forever, Mists, Era or Anniversary; those profiles'
sizing behavior is unchanged.
