# Øving 2

## Kjøring av koden

## Krav

- [Rust og Cargo](https://www.rust-lang.org/tools/install) må være installert.

## Kjør programmet

Åpne terminal i mappen og kjør:

```rust
cargo run
```

Eventuelt:

```rust
cargo build && cargo run
```

## Asymptotisk analyse

Metode 1 har en tidskompleksitet på Θ(n). For hvert rekursivt kall reduseres eksponenten med 1, og det gjøres kun en multiplikasjon per kall. Siden rekursjonen alltid går nøyaktig n steg før den når basistilfelle (n = 1), er kjøretiden proporsjonalt med n. Derfor er tidskompleksisteten Θ(n).

Metode 2 har en tidskompleksitet på Θ(log n). Eksponenten halveres ved hvert rekursivt kall (ved partall direkte, ved oddetall etter å trekke fra 1). Dette betyr at antall rekursive kall er Θ(log n), siden man må halvere n omtrent log2(n) ganger for å nå 1. Siden arbeidet per kall er konstant, er total kjøretid Θ(log n).

## Tidsanalyse

Hver algoritme ble kjørt 1000 ganger for hver verdi av n, og gjennomsnittlig kjøretid ble målt i nanosekunder. Resultatene er vist i tabellen nedenfor:

```rust
n           linear (ns) logarithmic (ns) builtin (ns)
-------------------------------------------------------
10                   48               14           36
100                 842               20           25
1000              10322               30           31
5000              50043               33           24
10000             41011               35           24
```

Ovenfor kan du se at metode 1 faktisk tar lengre tid enn metode 2 for store verdier av n, som forventet. Dette støtter teorien om at metode 2 er mer effektiv enn metode 1 på grunn av dens bedre O-notasjon. Det er også verdt å merke at builtin bruker cirka like mye tid som metode 2, noe som tyder på at Rust sin innebygde funksjon er optimalisert for større n.
