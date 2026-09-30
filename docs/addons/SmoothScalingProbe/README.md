# SmoothScalingProbe

Future **native recorder**, not native evidence. `/smoothscalingprobe` creates a visible comparison panel and records `SmoothScalingProbeDB` with build/display provenance, public creation default, XML omitted/true/false values, single/newline/wrapped text heights, font 12 with frame scale 1.1 and text scale 1.1, explicit fractional font size, and animation-mode independence. Repeated false/true/false samples expose stale layout.

1. Install on a matching retail client; adjust TOC interface to the actual client. No deployment has been performed.
2. Run `/smoothscalingprobe` out of combat. Review recorded errors rather than assuming unsupported calls worked.
3. Capture a screenshot at each physical display/UI scale being investigated. Rows label requested modes; SavedVariables cannot establish rendered glyph positions by themselves.
4. `/reload` or log out, then collect `WTF/Account/<ACCOUNT>/SavedVariables/SmoothScalingProbe.lua` plus screenshot and actual build.

Compare omitted/default versus explicit false, fractional height changes, wrapped line counts and visual baselines, frame-scale versus text-scale behavior, XML values, and mode flips. Determine native rounding direction/metric and whether public default matches XML default before replacing inferred simulator policy. This recorder does not exercise native secret booleans or establish security parity.
