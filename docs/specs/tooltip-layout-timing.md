# Tooltip layout timing

For an owned tooltip with content, adding a line or double line and changing
custom word-wrap width, custom line spacing, shrink-to-fit, or minimum width
must not synchronously resize it. A subsequent `Show` sizes its current content.
`SetPadding` synchronously lays out the current content. Repeated `Show` also
performs layout. Native evidence: WM63-L4 and WM63-L23, Retail 12.1.5.70688.

These timing rules do not establish native font metrics, markup widths, or
money and status-bar heights. The simulator's font sizes and line-height
multiplier are estimates, not native measurements. Geometry and padding side
semantics are outside this timing slice.
