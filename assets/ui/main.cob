#scenes
"main_scene"
    AbsoluteNode{left: 50% top:5%, flex_direction: Column}
    MainInterface
    "cell"
        Animated<BackgroundColor>{
            idle: #FF0000
            hover:Hsla{ hue:120 saturation:1.0 lightness: 0.50 alpha:0.2 }
            press:Hsla{ hue:240 saturation:1.0 lightness: 0.50 alpha:0.2 }
        }
        NodeShadow{ color: #000000FF spread_radius:10px blur_radius:5px  }
        "text"
            TextLine{text:"Hello, World!"}


"number_text"
    "cell"
        "text"
            TextLine{text:"placeholder"}

"exit_button"
    TextLine{text:"Exit"}
"despawn_button"
    TextLine{text:"Despawn"}
"respawn_scene"
    AbsoluteNode{left: 50% top:5%, flex_direction: Column}
    "respawn_button"
        TextLine{text:"Respawn"}
