# Instrukser for kjøring

## Krav

- [Rust og Cargo](https://www.rust-lang.org/tools/install) må være installert.

## Bygg programmene

Åpne terminal i `ov4`-mappen og kjør:

```
cargo build
```

Dette bygger begge programmene, `langetall` og `soketre`, siden de er definert som to separate `[[bin]]`-mål i samme `Cargo.toml`.

## Kjør Langetall (deloppgave 1)

```
cargo run --bin langetall -- <tall1> <+|-> <tall2>
```

Eksempel:

```
cargo run --bin langetall -- 100000000019999999999001 + 100007
```

## Kjør Søketre (deloppgave 2)

```
cargo run --bin soketre -- ord1 ord2 ord3 ...
```

Eksempel:

```
cargo run --bin soketre -- hode bein hals arm tann hånd tå
```

Du kan også kjøre programmet og skrive inn ordene:
```
cargo run --bin soketre
```