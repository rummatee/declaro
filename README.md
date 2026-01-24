<div align="center">

Declaro
===

A graphical user interface for the Nix language built with Dioxus and nil

</div>

# About
Declaro is a graphical user interface for [Nix](https://nixos.org).
It is built with [Dioxus](https://dioxuslabs.com/) to enable it to run on various platforms and on the web.
It uses [nil](https://github.com/oxalica/nil) to parse and analyze nix expression.
Currently this project is still in a PoC state with many vital features still missing.

## Why?
Nix is a very powerful system to write declarative and reproducible system configurations.
Nix based configurations can power various systems from NeoVim over your user configuration to the whole operating system and they can always easily be rolled back.
As Nix is Turing complete there are no limitations to what can be configured and how configuration aspects can be reused.
Additionally Nix features the biggest package store in existance.
The drawback of this power is that Nix can be quite hard to learn, especially for non-technical users.
There have been attempts in the past to make GUIs for Nix, mostly focusing on NixOS specifically, the most prominent example is [nixos-conf-editor](github.com/snowfallorg/nixos-conf-editor/).
However, these interfaces don't expose most features of the Nix language such as let-in expressions or lambdas, leading to the inability to reuse configuration logic in multiple places.
While this makes getting started easier for new users, it doesn't introduce them to some of the main reasons to use Nix.
This project aims to be a complete graphical programming language that can capture all elements of the Nix language for any target system.

# Trying it out
On a flakes enabled system enter a dev shell with
```
nix develop
```
and then start the application with
```
dx run
```

# Features

## Implemented
- Open and save files
- Edit simple strings, variable references, attribute-sets and show lambdas
- Change the type of expressions


## Roadmap
- Edit lambda parameters, edit, add and remove attribute-set paths, string interpolation
- Support lists, with- and let-expressions, numbers, booleans and paths
- Show available attributes, defaults and help for NixOS modules and flakes
- Visualize diffs between two Nix configurations
- Undo and Redo history


## License

This project is licensed under either the [MIT license] or the [Apache-2 License].

[apache-2 license]: https://github.com/DioxusLabs/dioxus/blob/master/LICENSE-APACHE
[mit license]: https://github.com/DioxusLabs/dioxus/blob/master/LICENSE-MIT
