# Bounce Up!
This is a game made during the 2021 summer jam. It is written in [rust](https://www.rust-lang.org/) and is powered by [bevy engine](https://bevyengine.org/).

Live version is available [here](https://cryscan.itch.io/bounce-up).

Thanks [@fplust](https://github.com/fplust) for porting the game to bevy v0.8!

Thanks [@Bobox214](https://github.com/Bobox214) for the [star background](https://github.com/Bobox214/Kataster)!

Sound effects obtained from [Zapslat](https://www.zapsplat.com).
Background music credits to the [Ultimate MIDI Pack](https://archive.org/details/ultimidi/) (License: CC-BY-SA).

## Build for Web
Run the following command to compile:
```shell
$ wasm-pack build --release --target web
```

Then add the `index.html` into the `pkg` folder:
```html
<html>

<head>
    <title>Bounce Up!</title>
</head>

<body>
    <script type="module">
        import init from "./bounce_up.js";
        init("./bounce_up_bg.wasm").then(function (wasm) {
            wasm.run();
        });
    </script>
</body>

</html>
```

Finally copy the `assets` folder into the `pkg` folder, pack and release.
