# Prp: Pure Rust Pip (Mostly)

<p align="center">
<img src="https://img.shields.io/crates/l/prp.svg" alt="license">
<a href="https://crates.io/crates/prp">
<img src="https://img.shields.io/crates/v/prp.svg?colorB=319e8c" alt="Version info">
</a>
<a href="https://github.com/DanCardin/prp/actions?query=workflow%3ATest">
<img src="https://github.com/DanCardin/prp/workflows/Test/badge.svg" alt="Build Status">
</a> <a href="https://codecov.io/gh/DanCardin/prp">
<img src="https://codecov.io/gh/DanCardin/prp/branch/main/graph/badge.svg?token=U7NQIWXWKW"/>
</a><br>
</p>

`prp` is a python project workflow tool, meant to replace all of the natural
uses of `pip`, `venv`, `pipx`, or other such tools.

It is **not** a dependency management/packaging tool like `poetry` or `flit`.

- If you use `venv` today, then the intent is that `prp` **should** feel
  familiar, but ideally a lot more streamlined. This is one of `prp`'s primary
  user targets.

- If you use `pipx`, `prp x` should do much the same thing (probably not "drop
  in" replaceable in general, but certainly for the common case of
  `pipx install foo`)!

- If you use `pip`, it should feel identical. It's **meant** to be a drop-in
  replacement, in terms of CLI options and behavior.

  (**note** "pure rust" is aspirational, it currently (and may forever) **does**
  call out to `pip` for the more involved commands, like `prp install`, which
  require reimplementing large swathes of python's packaging and building logic.
  Long term/ideally, it would be able to reproduce pip's behavior in rust
  directly).

[Scroll down](#comparison) for a comparison to other tools.

## Quickstart

Either download a binary from github, or run `cargo install prp` (...if you have
cargo).

Enable shell integration (i.e. virtualenv activation) by
`eval "$(prp shell init)"` (bash, zsh)

## What is `prp`

You can think of `prp` as an idea of what `pip` **could** be/do (Its name
intentionally phonetically sounds like pip even!). If it does not support some
option/command of `pip`, that should be considered a missing feature. The intent
is very much for it to be a drop in replacement for `pip`.

The differences in behavior are primarily oriented around imagining what `pip`
might be like if it were more like `npm` or `cargo`.

The core differences are:

- Virtual environments are automatically created

  `prp`, `prp venv`, `prp install`, etc all automatically create the virtual
  environment, according to your
  [configured strategy](#virtual-environment-strategies).

- Commands are local

  `prp install`, etc operate on the local virtual environment rather than
  globally (i.e. `pip`)

## Workflow Commands

### Global options

- `--shell zsh/bash/fish`

  Note, if you have `$SHELL` exported, or you've hooked into the shell with
  `prp --shell <shell> shell init`, you should never need to use this option.

- `-n/--name <name>`

  Controls the virtual environment name. If not used, the venv name will default
  to the global setting name (defaulting to `.venv`).

  However some people utiliize multiple virtualenvs for the same project (to
  test different branches or versions of python). For these people, this setting
  would be how you control which venv is used for various commands.

### `prp`

`prp`, with no subcommand is equivalent to `prp venv && prp activate`.

Note (as with all commands), this takes into account the global `-n/--name`
flag, so `prp -n foo`, `prp -n bar` can be used to quickly swap between venvs
with different names on the same project.

### `prp venv`

`prp venv` will automatically create a venv (if one does not exist).

Note that this does **not** internally invoke python's `venv` library/cli,
instead constructing the venv directly.

### `prp activate`

`prp activate` is equivalent to running `source .venv/bin/activate` on a normal
environment. Although note that `.venv/bin/activate` does not exist.

As such, in order for this to work, `prp` needs to have been hooked into the
shell with your shell-specific use of `prp shell init`

### `prp run`

`prp run` is roughly equivalent to `npm run` or `cargo run`, in that it
specifically runs virtual env scripts/binaries in the context of the venv
(whether activated or not).

As such, this could be used to run your project's entrypoint scripts, or
project-specific tooling installed within the venv.

### `prp exec`

`prp exec` can be used to run any cli command in the context of the virtual env.

### `prp shell`

`prp shell` is a group of shell-related subcommands.

- `prp shell` with no subcommend creates a new sub-shell with the virtual env
  already activated.

- `prp shell init` can be used (for example with bash/zsh
  `eval "$(prp shell init)`), to hook into the current/selected shell's
  execution to enable features like `prp activate`.

- `prp shell completion` can be used to write output shell completions for
  `prp`.

## Pip Commands

Native pip commands like `prp install`, `prp download`, etc can be invoked and
**should** generally work exactly like `pip` would, with the exception that
`prp` will not attempt to i.e. install a package globally.

Instead, whether inside a virtual environment or not, `prp install` should be
equivalent to `prp venv && prp activate && pip install`; that is, it will create
a venv if necessary, activate it, and perform the requested installation.

Note that **currently** such commands internally **do** invoke `pip` itself.
However, as ["Why Rust?"](#why-rust) implies, this isn't ideal. Ideally `prp`
would gradually internally replace `pip` invocations to less-python-dependent
options.

## Config

### Virtual Environment Strategies

Configured by `$XDG_CONFIG_HOME/prp.toml` -> `strategy = "<value>"`.

Given the automatic creation of virutal environments by `prp`, it's important
that it knows how you prefer to manage your virtual environments.

There are two strategies:

- `local` (default): i.e. in your project directory.

- `central`: Venvs are organized under `$XDG_DATA_HOME/prp/`, with a folder
  structure that mirrors the target directory. (i.e. `~/projects/foo` ->
  `$XDG_DATA_HOME/prp/projects/foo/.venv`).

With any strategy, `prp` will search upwards from the current directory for
"project" indicators (`pyproject.toml`, `setup.py`, `setup.cfg`, `.gitignore`)
and use the resultant directory as the target for the virtual environment. If
there is no such indicator, it falls back to the current directory.

### Venv name

Configured by `$XDG_CONFIG_HOME/prp.toml` -> `venv_name = "<name>"`.

Defaults to `.venv` if unset.

## Why Rust?

Hot take: Python is simply not ideal for producing a tool like this.

- This sort of tool being dependent on a valid python environment to function
  would be a thorn in its side.

  `poetry`, a popular python packaging tool has a complex installer system for
  ensuring that it's installed in an isolated manner. And even **that** can
  become broken.

- Speed/startup-wise python is also less than ideal for a tool like this. Being
  invoked frequently, the startup time for python would add
  unavoidable/unnecessary latency to every command.

By contrast this Rust tool ships a standalone binary, that operates basically
instantly in all cases.

Not being written in python, this tool can be used to bootstrap python
environments without requiring a functional env for the tool itself.
Perhaps/ideally even the python installation could be bootstrapped (using posy's
pybi ideas).

## Comparison

- `venv` (Python): The `venv` library/CLI tool **only** creates the virtual env.
  You still need tools like `pip`, `pipx`, `venv/bin/activate`/`deactivate`, etc
  to work effectively.

  `prp` intends to be more of a wholistic workflow tool. `prp` encodes your
  management preference as a setting, such that you should be able to just run
  `prp` (sans arguments), to create a virtual env (if it doesnt exist), and
  activate it in one go. In fact, `prp` does not produce a `bin/activate`
  script, because it's unnecessary.

- `pipx` (Python): `pipx` is actually a really nice tool! It's workflows are
  very much included/duplicated on purpose because they're done very nicely!

  What is unfortunate about `pipx`...is that it's written in python. Its whole
  purpose is to install python tools in isolated environments. This creates a
  "sort of", chicken and egg problem where your tool you use to install python
  tools would ideally have `pipx` available to install itself.

  Further, it puts `pipx` at a disadvantage, in that it depends upon the
  installing python's version to be compatible with the supported versions of
  itself.

  Being written in Rust, `prp` mostly just doesn't have `pipx`'s disadvantages.
  With regard to the `prp x` subset of commands, it **should** mostly function
  the same.

- [Rye](https://github.com/mitsuhiko/rye) (Rust): Seems most similar to `prp` of
  all the options. However it does dependency locking (with pip-tools), makes
  binary shims, and is altogether more opinionated.

  While `prp` intends to replace uses of `pip`/`venv` (and perhaps even
  installation of python itself) like `rye`, it does **not** intend to bake
  itself into a project in any way.

  My use of `prp` for managing python related projects has no visible effect on
  a project, because it's purely a workflow tool.

- [Posy](https://github.com/njsmith/posy) (Rust): Also replaces `pip` and
  virtualenvs, but does dependency locking and otherwise implies that you should
  define your dependencies in terms of posy.

  For much the same reasons as for `rye`, `prp` is different in that it does not
  affect the projects on which its used.

- [Poetry](https://github.com/python-poetry/poetry)/[Pipenv](https://github.com/pypa/pipenv)/[Flit](https://github.com/pypa/flit)/etc
  (Python):

  All these python projects are packaging/dependency management tools. While
  most of them also have virtual env management features, their primary purpose

  As such, it's 100% possible to use them in tandem with `prp`.
