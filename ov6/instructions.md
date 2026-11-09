# Instrukser for kjøring

## Krav

- [Rust og Cargo](https://www.rust-lang.org/tools/install) må være installert.

## Bygg programmene

Åpne terminal i `ov6`-mappen og kjør:

```
cargo build --release
```

Dette bygger begge programmene, `komprimering` og `dekomprimering`, siden de er definert som to separate `[[bin]]`-mål i samme `Cargo.toml`. Bruk `--release`, ellers blir store filer trege.

## Komprimer

```
cargo run --release --bin komprimering -- <innfil> [utfil]
```

Uten `utfil` skrives resultatet til `<innfil>.komp`.

Eksempel:

```
cargo run --release --bin komprimering -- diverse.lyx
```

## Dekomprimer (egen kjøring)

```
cargo run --release --bin dekomprimering -- <innfil.komp> [utfil]
```

Uten `utfil` skrives resultatet til `<innfil>.dekomp` (`.komp` fjernes fra navnet).

Eksempel:

```
cargo run --release --bin dekomprimering -- diverse.lyx.komp
```

## Sjekk at filene er like

Windows:

```
fc /b diverse.lyx diverse.lyx.dekomp
```

Linux/mac:

```
cmp diverse.lyx diverse.lyx.dekomp
```

## Hvordan det virker

1. **LZ77** finner gjentatte strenger opptil 1 MB bakover (lengde 3–32 770) med hashkjeder. Valget mellom bokstaver og referanser gjøres med optimal parsing: billigste vei i antall bits, regnet ut fra huffmankodene.
2. **Huffman** koder resultatet med to trær per blokk (1 MB), samme oppsett som deflate:
   - ett tre for bytes, lengdekoder og END-symbolet (313 symboler)
   - ett tre for avstandskoder (40 symboler)

   Lavere bits av lengder og avstander skrives ukomprimert som ekstrabits.
3. Kodene er kanoniske, så bare kodelengdene lagres i blokkhodet, kjørelengdekodet og huffmankodet som i deflate. Utpakkeren bygger samme koder ut fra dem.

Filformat: `u64` originalstørrelse, deretter en bitstrøm med blokker (1 bit «siste blokk», blokkhode, symboler, END). Alt dekomprimeringen trenger ligger i den komprimerte fila.

Komprimeringen bruker alle prosessorkjernene (én blokk per tråd). enwik8 tar ca. 1 minutt å komprimere og ca. 1 sekund å pakke ut.
