// All " within the layout definition need to be escape
pub const FALLBACK_LAYOUT: &str = "---
outlines:
    default: { width: 35.33, height: 52 }
    altline: { width: 52.67, height: 52 }
    wide: { width: 62, height: 52 }
    spaceline: { width: 142, height: 52 }
    special: { width: 44, height: 52 }

views:
    base:
        - \"q w e r t y u i o p\"
        - \"a s d f g h j k l\"
        - \"Shift_L   z x c v b n m  BackSpace\"
        - \"show_numbers preferences         space        period Return\"
    upper:
        - \"Q W E R T Y U I O P\"
        - \"A S D F G H J K L\"
        - \"Shift_L   Z X C V B N M  BackSpace\"
        - \"show_numbers preferences         space        period Return\"
    numbers:
        - \"1 2 3 4 5 6 7 8 9 0\"
        - \"@ # $ % & - _ + ( )\"
        - \"show_symbols   , \\\" ' colon ; ! ?  BackSpace\"
        - \"show_letters preferences         space        period Return\"
    symbols:
        - \"~ ` | · √ π τ ÷ × ¶\"
        - \"© ® £ € ¥ ^ ° * { }\"
        - \"show_numbers_from_symbols   \\\\ / < > = [ ]  BackSpace\"
        - \"show_letters preferences         space        period Return\"
";
