
# 3D RPG Bevy game template (WIP)
<table border="0">
    <tr>
        <td>
            <video src="https://github.com/user-attachments/assets/6533b1d9-5971-41fd-acc4-5d2f567266be" controls="controls"> </video>
        </td>
        <td>
            <video src="https://github.com/user-attachments/assets/3aadf8fc-7bb2-477d-82c6-44dc1d56ed08" controls="controls"> </video>
        </td>
    </tr>
    <tr>
        <td>
            <video src="https://github.com/user-attachments/assets/7f470899-29b0-44c5-b418-6fdf240d130c" controls="controls"> </video>
        </td>
        <td>
            <video src="https://github.com/user-attachments/assets/ea236931-6c59-456f-98de-447d3f6eb287" controls="controls"> </video>
        </td>
    </tr>
</table>

This template is based on the awesome [BevyFlock 2D template][BevyFlock] featuring out of the box builds for:
- Windows
- Linux
- macOS
- Web (Wasm)
This template is a great way to get started if you aim to build new 3D RPG [Bevy] game!
It is not as simple as bevy_new_2d which is aimed to an easy start. It focuses to have a rather solid structure to be able to carry the weight of big projects and tries to follow the [flat architercture](#project-structure) principle.
Start with a [basic project](#write-your-game) and [CI / CD](#release-your-game) that can deploy to [itch.io](https://itch.io).
You can [try this template in your browser!](https://olekspickle.itch.io/bevy-3d-rpg)

## Best way to start
Install [cargo-generate] or [bevy_cli] and run:
```bash
cargo generate olekspickle/bevy_new_3d_rpg -n my-rpg
# or with bevy_cli
bevy new -t=olekspickle/bevy_new_3d_rpg my-rpg
```

### Hotpatching
If you want to use serving with hotpatching, you can use dioxus-cli:

- Linux: `make hot` or `bash BEVY_ASSET_ROOT="." dx serve --hot-patch`
- Windows PS:`$env:BEVY_ASSET_ROOT="." ; dx serve --hot-patch`

## Features:
- [x] flat cargo workspace based project structure for game logic crates that can grow and be maintainable
- [x] import and usage of game mechanics and parameters from .ron (config, credits) (kudos to Caudiciform)
- [x] simple asset loading based on [bevy_asset_loader] with loading from path addition (kudos to Caudiciform)
- [x] third person camera with [bevy_third_person_camera]
- [x] solid keyboard & gamepad mapping to ui & game actions using [bevy_enhanced_input]
- [x] simple scene with colliders and rigid bodies using [avian3d]
- [x] simple player movement using [bevy_tnua]
- [x] simple skybox sun cycle using [bevy atmosphere example], with daynight and nimbus modes
- [x] featuring rig and animations using [Universal Animation Library] from quaternius
- [x] experimental sound with [bevy_seedling] based on Firewheel audio engine (which will probably replace bevy_audio), with **highly** experimental audio stutter fix for web
- [x] consistent Esc back navigation in gameplay and menu via stacked modals (kudos for the idea to skyemakesgames)
- [x] serialize and save settings
- [x] audio, video and keys rebind tabs in settings (currently not really working)
- [x] easy drop in scene integration using awesome [skein] with a simple scene

### TODOs
- [ ] add basic mood change per zone
- [ ] implement different music states(exploration, combat)
- [ ] custom font replace example using pre-loaded font
- [ ] sky background instead of just void lol
- [ ] Movement sfx: jump, dash, sprint
- [ ] spatial audio demo: boombox emitting background music
- [ ] Jump with timer(tricky with tnua jump in air counter)
- [ ] small door/portal demo
- [ ] split screen for coop
- [ ] vault on objects if they are reachable
- [ ] climbing
- [ ] do not rotate player on aim(silly bug, check it out - release aim looking to the floor)
- [ ] basic fighting: punch, kick, take weapon
- [ ] weapon select wheel
- [ ] bow
- [ ] rifle

## Write your game

The best way to get started is to play around with the code you find in [`src/game/`](./src/game).
This template comes with a basic project structure that you may find useful:

### Project structure
| Path                                                  | Description                                                           |
| ----------------------------------------------------- | --------------------------------------------------------------------- |
| [`src/main.rs`](./src/main.rs)                        | App entrypoint where system plugins and window set up                 |
| [`assets`](./assets)                                  | Asset directory                                                       |
| [`crates/asset_loading`](./src/crates/asset_loading)  | A high-level way to load collections of asset handles as resources    |
| [`crates/models`](./src/crates/models)                | Data source for the game: inputs, markers, timers                     |
| [`crates/audio`](./src/crates/audio)                  | Marker components for sound effects and music, bus setup              |
| [`crates/screens`](./src/crates/screens)              | Splash/title/gameplay and other screen related systems and ui         |
| [`crates/scene`](./src/crates/scene)                  | Scene setup, skybox                                                   |
| [`crates/game`](./src/crates/game)                    | Game mechanics & content                                              |
| [`crates/player`](./src/crates/player)                | Player control & animation                                            |
| [`crates/ui`](./src/crates/ui)                        | Reusable UI widgets & game color pallet control                       |

Feel free to move things around however you want, though

## Run your game

### Makefile
There are some helpful commands in [Makefile](./Makefile) to simplify build options
But generally running your game locally is very simple:

<details>
    <summary><ins>Bevy CLI</ins></summary>

    - Dev: `bevy run` to run a native dev build
    - Release: `bevy run --release` to run a native release build
- Use `bevy run --release web` to run a web release build
To run a **web** dev build to run audio in separate thread to avoid audio stuttering:
- :`bash bevy run web --headers="Cross-Origin-Opener-Policy:same-origin" --headers="Cross-Origin-Embedder-Policy:credentialless" `
</details>

<details>
    <summary><ins>CMake</ins></summary>

    - Dev: `make run` to run a **native** dev build
- Release: `make build` to build a **native** release build
- Web: `make run-web` to run a **web** dev build to run audio in separate thread to avoid audio stuttering
</details>

<details>
<summary><ins>Installing Linux dependencies</ins></summary>

  If you're using Linux, make sure you've installed Bevy's [Linux dependencies].
  Note that this template enables Wayland support, which requires additional dependencies as detailed in the link above.
  Wayland is activated by using the `bevy/wayland` feature in the [`Cargo.toml`](./Cargo.toml).
</details>

<details>
<summary><ins>(Optional) Improving compile times</ins></summary>

[`.cargo/config.toml`](./.cargo/config.toml) contains documentation on how to set up your environment to improve compile times.
</details>

WARNING: if you work in a private repository, please be aware that macOS and Windows runners cost more build minutes.
**For public repositories the workflow runners are free!**

## Release your game

This template uses [GitHub workflows] to run tests and build releases.
Check the [release-flow](.github/workflows/release.yaml)

## Known issues

There are some known issues in Bevy that can require arcane workarounds.

### My audio is stuttering on web

This template uses firewheel experimental audio runnign in the separate worker thread, so it should not be happening, but if you experience it nevertheless, here are a few tips:
- If you're using materials, you should force your render pipelines to [load at the start of the game]
- Optimize your game as much as you can to keep its FPS high.
- Apply the suggestions from the blog post [Workaround for the Choppy Music in Bevy Web Builds].
- Advise your users to try a Chromium-based browser if there are still issues.

### My game window flashes white for a split second when I start the game on Windows

The game window is created before the GPU is ready to render everything.
This means that it'll start with a white screen for a few frames.
The workaround is to [spawn the Window hidden] and only [make it visible a few frames later]

### My character or camera movement is choppy

Choppy character movement is often caused by movement updates being tied to the frame rate.
See the [`physics_in_fixed_timestep`] example for how to fix this.

Choppy camera movement is almost always caused by the camera being tied too tightly to a moving target position.
You can use [`smooth_nudge`] to make your camera smoothly approach its target position instead.

## Credits

The [assets](./assets) in this repository are all 3rd-party. See the see [credits](assets/credits.json) for more information.

## License

The source code in this repository is licensed under any of the following at your option:
- [CC0-1.0 License](./LICENSE-CC0)
- [MIT License](./LICENSE-MIT)
- [Apache License, Version 2.0](./LICENSE-APACHE)

## Bevy Compatibility

| bevy | bevy_new_3d_rpg  |
| ---- | ---------------------- |
| 0.16 |       main,0.1.4       |

[avian3d]: https://github.com/Jondolf/avian/tree/main/crates/avian3d
[bevy]: https://bevyengine.org/
[bevy atmosphere example]: https://bevyengine.org/examples/3d-rendering/atmosphere/
[bevy-discord]: https://discord.gg/bevy
[bevy_asset_loader]: https://github.com/NiklasEi/bevy_asset_loader
[bevy_cli]: https://github.com/TheBevyFlock/bevy_cli
[bevy-learn]: https://bevyengine.org/learn/
[bevy_seedling]: https://github.com/CorvusPrudens/bevy_seedling
[bevy_third_person_camera]: https://github.com/The-DevBlog/bevy_third_person_camera
[bevy_tnua]: https://github.com/idanarye/bevy-tnua
[Bevy Cheat Book]: https://bevy-cheatbook.github.io/introduction.html
[BevyFlock]: https://github.com/TheBevyFlock/bevy_new_2d
[bevy_enhanced_input]: https://github.com/projectharmonia/bevy_enhanced_input
[cargo-generate]: https://github.com/cargo-generate/cargo-generate
[GitHub workflows]: https://docs.github.com/en/actions/using-workflows
[Linux dependencies]: https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md
[skein]: https://bevyskein.dev
[trunk]: https://trunkrs.dev/
[Universal Animation Library]: https://quaternius.itch.io/universal-animation-library

[spawn the Window hidden]: https://github.com/bevyengine/bevy/blob/release-0.14.0/examples/window/window_settings.rs#L29-L32
[make it visible a few frames later]: https://github.com/bevyengine/bevy/blob/release-0.14.0/examples/window/window_settings.rs#L56-L64
[`physics_in_fixed_timestep`]: https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs
[`smooth_nudge`]: https://github.com/bevyengine/bevy/blob/main/examples/movement/smooth_follow.rs#L127-L142
[load at the start of the game]: https://github.com/rparrett/bevy_pipelines_ready/blob/main/src/lib.rs
