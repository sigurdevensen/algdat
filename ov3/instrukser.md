# Instrukser for kjøring

## Krav
- [Rust og Cargo](https://www.rust-lang.org/tools/install) må være installert.

## Kjør programmet

Åpne terminal i mappen og kjør (viktig med `--release`, ellers blir 50 millioner tall veldig tregt å sortere):

```
cargo run --release
```

Programmet:
1. Kjører en rask sanity-sjekk på et lite datasett.
2. Genererer fire tabeller med 50 millioner tall hver: tilfeldige tall, mange
   duplikater (annenhvert element likt), sortert fra før, og baklengs sortert.
3. Sorterer hver tabell med både single-pivot quicksort og dual-pivot
   quicksort, tar tiden, og sjekker sjekksum og rekkefølge etter hver
   sortering (programmet stopper med feilmelding hvis en test feiler).
4. Skriver ut en oppsummeringstabell som viser hvilken variant som var
   raskest for hvert datasett.

`n` (antall tall) kan endres i `main.rs`, i `main()`-funksjonen.
