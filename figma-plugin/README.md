# Embedded GUI Sync (Figma Plugin)

Official Figma plugin to convert Figma designs, AutoLayout frames, components, and design systems directly into declarative KDL markup for `embedded-gui` and `embedded-gui-studio`.

## Installation into Figma

1. Open **Figma Desktop** (or Web).
2. Go to **Menu (Figma logo)** ➔ **Plugins** ➔ **Development** ➔ **Import plugin from manifest...**.
3. Select the `manifest.json` file inside this `figma-plugin/` directory.
4. The plugin **Embedded GUI Sync** is now available in your Figma workspace!

## How to Use

1. Design your embedded UI screen inside a Figma Frame (e.g. `320×240` or `480×320` px).
2. Use **AutoLayout** (Horizontal or Vertical flex) or group your widgets.
3. Name your layers/components intuitively:
   - `Button / Save` ➔ `button id="save" text="Save"`
   - `Slider / Brightness` ➔ `slider id="brightness" ...`
   - `Toggle / Power` ➔ `toggle id="power" ...`
   - `Progress / Battery` ➔ `progress id="battery" ...`
   - `Gauge / Speed` ➔ `scale id="speed" ...`
   - `Plotter / Wave` ➔ `plotter id="wave" ...`
4. Select the Frame in Figma.
5. Click **📋 Copy KDL for Embedded GUI**.
6. Paste into **`embedded-gui-studio`** or directly into your project's `.kdl` file!
