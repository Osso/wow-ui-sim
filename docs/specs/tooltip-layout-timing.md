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
and most text-anchor placement are outside this timing slice. For the first
line without attachments, Retail/PTR layout assigns TOP and LEFT points: the
first TOP is `-10 - top` from the tooltip; subsequent TOP points follow the
preceding indexed left line with the configured spacing. A hidden left line with a
known frame is omitted from the height sum; spacing is added only after a
nonzero accumulated height. Hidden lines still remain the indexed anchor
reference (WM72). LEFT is `10 + left` from the tooltip. Shown right lines anchor RIGHT to the same left line's LEFT
by the content width. A hidden right line retains its previous points until
shown (WM72). An unforced, unwrapped two-column row reserves 38.4 in
width selection (WM72). A nonempty right text disables a double line's
requested wrap; an empty right text leaves the requested flag unchanged (WM72).
Empty or absent text on both sides does not append a row. A space or nonempty
right text does append a row (WM72).
The renderer follows these matching anchors for
attachment-free rows. Attachment offsets, forced-width layout, and manually
reanchored rows outside those point pairs remain outside this slice. Ordinary
`GetPoint` publication is inferred, not observed. For Retail/PTR, outer width
and height add 20.48 plus their explicit sides after content sizing. Padding
sides are right, bottom, left, top; omitted left and top preserve their prior
values. This arithmetic follows WM63-L23 (Retail 12.1.5.70688). Render and periodic
update do not recalculate Retail/PTR content size; they resolve stored geometry.
WM63-L4 line 27 verifies that the native periodic-update override does not call
the content-sizing routine. WM63 is evidence for
Retail 70688, not Wrath, Forever, Mists, Era or Anniversary; those profiles'
sizing behavior is unchanged.
