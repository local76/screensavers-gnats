# gnats

> A swarm of fireflies in your system accent, drifting around a triadic wireframe.

A boid-style swarm of 30–60 fireflies in a triadic palette, drifting through a wireframe network over a starfield. Predators and prey create natural clusters; the swarm reacts to system load.

## Visual elements

- **Swarm**. 30–60 firefly particles, each with a triadic color tint (accent + accent-hot + accent-cool).
- **Wireframe**. A triadic wireframe network drawn behind the swarm.
- **Starfield**. Subtle twinkling stars.
- **Predator / prey dynamics**. A small subset of gnats are flagged as predators and pursue the rest; the rest flee. The result is natural-looking clusters and chases.

## Dynamic / live behavior

- **Live hostname in the title**. The screensaver window title is suffixed with the host's hostname.
- **System load reactions**. Higher CPU/memory pressure increases swarm velocity and panic. The swarm gets edgier under load.
- **Per-machine personality**. `host_bias` slightly shifts preferred swarm neighborhoods per computer.
- **Accent color**. Gnats are tinted by the system accent (rotating among accent / accent-hot / accent-cool).

## Configuration (registry)

Under `HKEY_CURRENT_USER\Software\local76\gnats`:

- `SwarmSize`: 30–60 (clamped). Number of active fireflies.
- `PredatorRatio`: 0.0–0.3. Fraction of the swarm flagged as predators.

Global:

- `ColorTheme`, `GlobalScanlines` apply.

## Notes

- New to the collection. One of the more organic, ambient scenes.
- Looks great on multi-monitor setups.
- The boid math is deterministic — same seed produces the same swarm.

Part of the [screensavers](https://github.com/local76/screensavers) collection. See the root README for installation.
