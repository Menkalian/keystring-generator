# keystring_generator
[![Crates.io][crates-badge]][crates-url]
[![Build Status][ci-badge]][ci-url]
[![MIT licensed][mit-badge]][mit-url]

[ci-badge]: https://github.com/menkalian/keystring-generator/workflows/Rust/badge.svg
[ci-url]: https://github.com/menkalian/keystring-generator/actions
[crates-badge]: https://img.shields.io/crates/v/keystring_generator.svg
[crates-url]: https://crates.io/crates/keystring_generator
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/menkalian/keystring-generator/blob/master/LICENSE

This is a tool to generate rust code with hierarchical string constants from a simple file.

## Usage

The library exposes two functions: 

`generate(input: &PathBuf) -> Result<(), String>`

and

`generate_with_config(input: &PathBuf, output_dir: Option<&PathBuf>, ) -> Result<(), String>`

Please look in the documentation for `generate_with_config` to see an explanation for the parameters. Calling these methods will create a file `constants.rs` in the output directory (default: `generated/keygen`). This file has to be included in your project to be used.

## Input format
There are two variants of the input format: hierarchical or enumerated.

The hierarchical variant is simmilar to yaml and based on indentations.

**You are ABLE to mix tabs and spaces for indentations, but it is NOT RECOMMENDED.
If these are used simulaneously a tab is treated like 4 spaces!
So please just use one or the other.**

An example for a hierarchical input file looks like this:

````
hierarchical
  keys
    with
      five
        layers
      six
        hierarchical
          layers
````

The enumerated variant lists all the desired keys with `.` as the separator and looks like this:

````
hierarchical.keys
hierarchical.keys.with.five.layers
hierarchical.keys.with.six.hierarchical.layers
````

(Redundant enumeration of subkeys is possible but not necessary).

You may also mix these variants by creating an input file like this:
````
hierarchical.keys.with
  five.layers
  six
    hierarchical
      layers
````

## Output format

The output file for the above input will look (syntactically) like this:
````rust
pub mod hierarchical {
    const _BASE : &str = "hierarchical";
    pub mod keys {
        pub const _BASE: &str = "hierarchical.keys";

        pub mod with {
            pub const _BASE: &str = "hierarchical.keys.with";

            pub mod five {
                pub const _BASE: &str = "hierarchical.keys.with.five";
                pub const layers: &str = "hierarchical.keys.with.five.layers";
            }

            pub mod six {
                pub const _BASE: &str = "hierarchical.keys.with.six";

                pub mod hierarchical {
                    pub const _BASE: &str = "hierarchical.keys.with.six.hierarchical";
                    pub const layers: &str = "hierarchical.keys.with.six.hierarchical.layers";
                }
            }
        }
    }
}
````

You may then create a file `src/constants.rs` in your project with the following content:
````rust
include!("../generated/keygen/keygen.rs");
````

and include that in your `src/lib.rs` or `src/main.rs` file by declaring:
````rust
mod constants;
````

Therefore you can use the keys like this `constants::hierarchical::keys::with::five::layers` or `constants::hierarchical::keys::_BASE`.
