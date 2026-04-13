use chumsky::text::Char;
use std::fmt::Display;
use std::ops::Index;

#[derive(Clone, PartialEq)]
pub struct CodeColour {
    foreground: HslColour,
    background: HslColour,
    tooltip: Option<String>,
}

impl CodeColour {
    pub fn black_on_white() -> Self {
        Self {
            foreground: HslColour::grey(0.0),
            background: HslColour::grey(1.0),
            tooltip: None,
        }
    }
}

pub struct CodeDocument {
    text: String,
    colours: Vec<CodeColour>,
}

impl CodeDocument {
    pub fn new(text: String) -> Self {
        let length = text.len();

        Self {
            text,
            colours: vec![CodeColour::black_on_white(); length],
        }
    }

    pub fn apply_highlighting(
        &mut self,
        highlighting: &SyntaxHighlighting,
        ramps: &ColourRampCollection,
    ) {
        for highlight in &highlighting.highlights {
            for i in highlight.from..highlight.to {
                self.colours[i].background = highlight.colour.to_hsl(ramps);
                self.colours[i].tooltip = Some(highlight.tooltip.clone());
            }
        }
    }

    pub fn to_html(&self) -> String {
        let mut output = Vec::new();

        output.push(
            "<!doctype html>
<html lang=en>
<head>
<meta charset=utf-8>
<title>Colour ramp demonstration page</title>
<style>p { margin: 0; } span { display:inline-block; padding:0.2em 0; }</style>
</head>
<body>
<div style=\"font-family: monospace, monospace\">"
                .to_string(),
        );

        output.push("<p>".to_string());

        let mut previous_style = None;
        for (i, character) in self.text.chars().enumerate() {
            if character.is_newline() {
                if previous_style.is_some() {
                    output.push("</span>".to_string());
                } else {
                    // Make sure empty lines have the same height as a normal line:
                    output.push("<span>&nbsp;</span>".to_string());
                }
                output.push("</p>\n<p>".to_string());
                previous_style = None;
            } else {
                let style = &self.colours[i];
                if Some(style) != previous_style {
                    if previous_style.is_some() {
                        output.push("</span>".to_string());
                    }
                    let tooltip = match &style.tooltip {
                        None => "".to_string(),
                        Some(tooltip) => {
                            format!(" title=\"{}\"", tooltip.replace("\n\n", "\n")) // Markdown needs two newlines, whereas an HTML tooltip only needs one
                        }
                    };
                    output.push(format!(
                        "<span style=\"color:{};background-color:{}\"{}>",
                        style.foreground.to_hex(),
                        style.background.to_hex(),
                        tooltip
                    ));
                    previous_style = Some(style);
                }
                if character.is_whitespace() {
                    output.push("&nbsp;".to_string());
                } else {
                    output.push(character.to_string());
                }
            }
        }

        output.push("</p>".to_string());

        output.push(
            "</div>
</body>«
</html>"
                .to_string(),
        );

        output.join("")
    }
}

pub struct SyntaxHighlighting {
    highlights: Vec<Highlight>,
    pub overview: String,
}

impl SyntaxHighlighting {
    pub fn new() -> Self {
        Self {
            highlights: Vec::new(),
            overview: "".to_string(),
        }
    }

    pub fn add_highlight(&mut self, highlight: Highlight) {
        self.highlights.push(highlight);
    }

    pub fn set_overview(&mut self, overview: String) {
        self.overview = overview;
    }

    pub fn json<S1: Display, S2: Display>(
        &self,
        new_line: S1,
        indent: S2,
        ramps: &ColourRampCollection,
    ) -> String {
        let elements = self
            .highlights
            .iter()
            .map(|h| h.json(format!("{new_line}{indent}"), &indent, ramps))
            .collect::<Vec<_>>();
        format!(
            "[{new_line}{indent}{}{new_line}]",
            elements.join(&format!(",{new_line}{indent}"))
        )
    }
}

pub struct Highlight {
    from: usize,
    to: usize,
    colour: Colour,
    tooltip: String,
}

impl Highlight {
    pub fn new<S: Into<String>>(from: usize, to: usize, colour: Colour, tooltip: S) -> Self {
        Self {
            from,
            to,
            colour,
            tooltip: tooltip.into(),
        }
    }

    pub fn json<S1: Display, S2: Display>(
        &self,
        new_line: S1,
        indent: S2,
        ramps: &ColourRampCollection,
    ) -> String {
        pub fn round_float(value: f64) -> String {
            format!("{:.3}", value)
                .trim_end_matches("0")
                .trim_end_matches(".")
                .to_string()
        }

        let mut tooltip_with_grey = self
            .tooltip
            .replace("<grey>", "<span style=\"color:#888888;\">")
            .replace("</grey>", "</span>");

        while let Some(start_index) = tooltip_with_grey.find("<ColoredNumber>") {
            let end_index = start_index
                + tooltip_with_grey[start_index..]
                    .find("</ColoredNumber>")
                    .expect("Could not find matching `</ColoredNumber>` for `<ColoredNumber`)");
            let text_between =
                &tooltip_with_grey[(start_index + "<ColoredNumber>".len())..end_index];

            let (first, second) = text_between.split_once(",") .unwrap_or_else(|| panic!("`<ColoredNumber>` must contain an intensity between 0.0 and 1.0 and a colour ramp index, separated by a comma (e.g. `0.7, 2`). Received `{}`.",text_between) );
            let intensity = first.trim().parse::<f64>().unwrap_or_else(|e| {
                panic!(
                    "Cannot parse intensity of <ColoredNumber> colour: `{}`. Error: {}",
                    first.trim(),
                    e
                )
            });
            let ramp_index = second.trim().parse::<usize>().unwrap_or_else(|e| {
                panic!(
                    "Cannot parse colour ramp index of <ColoredNumber> colour: `{}`. Error: {}",
                    second.trim(),
                    e
                )
            });
            let bg_colour = Colour::new(ramp_index, intensity);
            let bg_hsl_colour = bg_colour.to_hsl(ramps);
            let bg_hex_colour = bg_hsl_colour.to_hex();

            let fg_hex_colour = if bg_hsl_colour.lightness < 0.3 {
                "#FFFFFF"
            } else {
                "#000000"
            };

            let replacement_range = start_index..end_index + "</ColoredNumber>".len();
            tooltip_with_grey.replace_range(
                replacement_range,
                &format!(
                    "<span style=\"color:{};background-color:{};\">&thinsp;{}&thinsp;</span>",
                    fg_hex_colour,
                    bg_hex_colour,
                    round_float(intensity)
                ),
            );
        }

        let json_tooltip = tooltip_with_grey.replace("\n", "\\n").replace("\"", "\\\"");
        format!(
            "{{{new_line}{indent}\"from\": {},{new_line}{indent}\"to\": {},{new_line}{indent}\"tooltip\": \"{}\",{new_line}{indent}\"colour\": \"{}\"{new_line}}}",
            self.from,
            self.to,
            json_tooltip,
            self.colour.to_hsl(ramps).to_hex()
        )
    }
}

pub struct Colour {
    group: usize,
    intensity: f64,
}

impl Colour {
    pub fn new(group: usize, intensity: f64) -> Self {
        Self { group, intensity }
    }

    pub fn to_hsl(&self, ramps: &ColourRampCollection) -> HslColour {
        ramps[self.group].sample(self.intensity)
    }
}

pub struct ColourRampCollection {
    ramps: Vec<ColourRamp>,
}

impl ColourRampCollection {
    pub fn new() -> Self {
        Self { ramps: Vec::new() }
    }

    pub fn with_predefined_ramps() -> Self {
        let colour_for_zero = HslColour::grey(0.9);
        Self {
            ramps: vec![
                ColourRamp::with_colour_for_zero(
                    vec![
                        ColourRampEntry::new(0.0, HslColour::new(50.0, 0.5, 0.9)),
                        ColourRampEntry::new(1.0, HslColour::new(-50.0, 0.9, 0.33)),
                    ],
                    colour_for_zero,
                ),
                ColourRamp::with_colour_for_zero(
                    vec![
                        ColourRampEntry::new(0.0, HslColour::new(80.0, 0.5, 0.9)),
                        ColourRampEntry::new(1.0, HslColour::new(140.0, 0.9, 0.2)),
                    ],
                    colour_for_zero,
                ),
                ColourRamp::with_colour_for_zero(
                    vec![
                        ColourRampEntry::new(0.0, HslColour::new(150.0, 0.5, 0.97)),
                        ColourRampEntry::new(1.0, HslColour::new(245.0, 0.9, 0.5)),
                    ],
                    colour_for_zero,
                ),
                ColourRamp::with_colour_for_zero(
                    vec![
                        ColourRampEntry::new(0.0, HslColour::new(315.0, 0.5, 0.97)),
                        ColourRampEntry::new(1.0, HslColour::new(270.0, 0.9, 0.45)),
                    ],
                    colour_for_zero,
                ),
            ],
        }
    }

    pub fn add_ramp(&mut self, ramp: ColourRamp) -> usize {
        let index = self.ramps.len();
        self.ramps.push(ramp);
        index
    }

    pub fn len(&self) -> usize {
        self.ramps.len()
    }
}

impl Index<usize> for ColourRampCollection {
    type Output = ColourRamp;

    fn index(&self, index: usize) -> &Self::Output {
        &self.ramps[index]
    }
}

pub struct ColourRamp {
    entries: Vec<ColourRampEntry>,
    colour_for_zero: Option<HslColour>,
}

impl ColourRamp {
    pub fn new(entries: Vec<ColourRampEntry>) -> Self {
        Self {
            entries,
            colour_for_zero: None,
        }
    }
    pub fn with_colour_for_zero(entries: Vec<ColourRampEntry>, colour_for_zero: HslColour) -> Self {
        Self {
            entries,
            colour_for_zero: Some(colour_for_zero),
        }
    }

    pub fn sample(&self, location: f64) -> HslColour {
        assert!(
            self.entries.len() >= 2,
            "Can only sample ColourRamp that contains at least two colours"
        );
        if location == 0.0
            && let Some(colour_for_zero) = self.colour_for_zero
        {
            colour_for_zero
        } else {
            let mut prev_entry = self.entries[0].clone();
            if location < prev_entry.location {
                panic!(
                    "Cannot sample ColourRamp outside of its specified range. Sampled at {}, but first entry is at location {}",
                    location, prev_entry.location
                )
            }
            for entry in self.entries.iter().skip(1) {
                if entry.location < location {
                    prev_entry = entry.clone();
                } else {
                    let distance = entry.location - prev_entry.location;
                    let progress = (location - prev_entry.location) / distance;
                    return prev_entry.colour.interpolate(&entry.colour, progress);
                }
            }
            panic!(
                "Cannot sample ColourRamp outside of its specified range. Sampled at {}, but last entry is at location {}",
                location, prev_entry.location
            )
        }
    }

    pub fn produce_example_page<P: AsRef<std::path::Path>>(
        &self,
        destination: P,
        from: f64,
        to: f64,
        step: f64,
    ) {
        let mut output = Vec::new();

        output.push(
            "<!doctype html>
<html lang=en>
<head>
<meta charset=utf-8>
<title>Colour ramp demonstration page</title>
</head>
<body>"
                .to_string(),
        );

        let mut location = from;
        while location <= to {
            let colour = self.sample(location);
            output.push(format!(
                "<div style=\"background-color:{};width:200px;height:30px\">{:.3}</div>",
                colour.to_hex(),
                location
            ));
            location += step;
        }
        output.push(
            "</body>
</html>"
                .to_string(),
        );

        std::fs::write(destination, output.join("\n")).expect("Failed to write sample page");
    }
}

#[derive(Clone)]
pub struct ColourRampEntry {
    location: f64,
    colour: HslColour,
}

impl ColourRampEntry {
    pub fn new(location: f64, colour: HslColour) -> Self {
        Self { location, colour }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct HslColour {
    hue: f64,
    saturation: f64,
    lightness: f64,
}

impl HslColour {
    pub fn new(hue: f64, saturation: f64, lightness: f64) -> Self {
        Self {
            hue: hue.rem_euclid(360.0),
            saturation,
            lightness,
        }
    }

    pub fn grey(lightness: f64) -> Self {
        Self {
            hue: 0.0,
            saturation: 0.0,
            lightness,
        }
    }

    pub fn interpolate(&self, other: &Self, progress: f64) -> Self {
        let anti_progress = 1.0 - progress;
        let hue = if (self.hue - other.hue).abs() <= 180.0 {
            self.hue * anti_progress + other.hue * progress
        } else if self.hue < other.hue {
            ((self.hue + 360.0) * anti_progress + other.hue * progress).rem_euclid(360.0)
        } else {
            (self.hue * anti_progress + (other.hue + 360.0) * progress).rem_euclid(360.0)
        };
        let saturation = self.saturation * anti_progress + other.saturation * progress;
        let lightness = self.lightness * anti_progress + other.lightness * progress;

        Self {
            hue,
            saturation,
            lightness,
        }
    }

    pub fn to_hex(&self) -> String {
        // https://www.rapidtables.com/convert/color/hsl-to-rgb.html
        let c = (1.0 - (2.0 * self.lightness - 1.0).abs()) * self.saturation;
        let x = c * (1.0 - ((self.hue / 60.0) % 2.0 - 1.0).abs());
        let m = self.lightness - (c * 0.5);
        let (r, g, b) = if 0.0 <= self.hue && self.hue < 60.0 {
            (c, x, 0.0)
        } else if 60.0 <= self.hue && self.hue < 120.0 {
            (x, c, 0.0)
        } else if 120.0 <= self.hue && self.hue < 180.0 {
            (0.0, c, x)
        } else if 180.0 <= self.hue && self.hue < 240.0 {
            (0.0, x, c)
        } else if 240.0 <= self.hue && self.hue < 300.0 {
            (x, 0.0, c)
        } else if 300.0 <= self.hue && self.hue < 360.0 {
            (c, 0.0, x)
        } else {
            panic!("Hue must be between at least 0.0 and less than 360.0")
        };

        fn float_to_hex(val: f64) -> String {
            format!("{:02x}", ((val * 256.0) as i64).clamp(0, 255))
        }

        let (r, g, b) = (r + m, g + m, b + m);

        format!("#{}{}{}", float_to_hex(r), float_to_hex(g), float_to_hex(b))
    }
}

#[cfg(test)]
mod tests {
    // This test generates sample html pages for the ramps provided by `with_predefined_ramps`

    // use crate::syntax_highlighting::{ColourRampCollection, HslColour};
    //
    // #[test]
    // fn test_sample_output() {
    //     let ramps = ColourRampCollection::with_predefined_ramps(HslColour::grey(0.9));
    //     for i in 0..ramps.len() {
    //         ramps[i].produce_example_page(format!("ramp_{}.html", i), 0.0, 1.0, 0.025);
    //     }
    // }
}
