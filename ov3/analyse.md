# Øving 3 - Alternativ 1: Sammenligne single-pivot og dual-pivot quicksort

## Implementasjon

**Single-pivot quicksort** (Sedgewick-varianten): tabellen stokkes tilfeldig før sortering starter, og deretter brukes vanlig 3-veis partisjonering med `arr[low]` som delingstall. Stokkingen gjør at rekkefølgen på data ikke har noe å si - algoritmen får forventet `O(n log n)` uansett om tabellen i utgangspunktet var tilfeldig, sortert eller baklengs sortert. 3-veis partisjonering gjør i tillegg at tabeller med mange like elementer får `O(n log n)` i stedet for `O(n^2)`, siden elementer lik pivot ekskluderes fra videre rekursjon.

**Dual-pivot quicksort**, basert på [geeksforgeeks](https://www.geeksforgeeks.org/dual-pivot-quicksort/), med de to fiksene oppgaveteksten krever:

1. Før partisjonering byttes `arr[low]` med `arr[low+(high-low)/3]`, og `arr[high]` byttes med `arr[high-(high-low)/3]`. Dette hindrer ekstrem skjevdeling (og dermed `O(n^2)`) når tabellen er sortert fra før.

2. Etter partisjonering: hvis de to delingstallene (pivotene) er like, gjøres det ikke noe rekursivt kall på midtintervallet, siden alle elementene der må være like pivotene og dermed allerede er ferdig sortert. Dette hindrer `O(n²)` på tabeller med mange like elementer.

Begge algoritmene er testet med sjekksum og rekkefølgetest (se `time_sort` i `main.rs`): sjekksummen av tabellen beregnes før og etter sortering og må stemme overens, og `tabell[i+1] >= tabell[i]` sjekkes for alle `i`. Testene kjøres for alle sorteringer i programmet, og programmet stopper (`assert!`) hvis en test feiler. I tillegg er det en liten sanity-sjekk på et håndkontrollerbart datasett før de store målingene.

## Asymptotisk analyse

Begge algoritmene deler tabellen i like store deler i det forventede tilfellet, med
`O(n)` arbeid på partisjonering per nivå. Mastersetningen på formen
`T(n) = a·T(n/b) + O(n)` gir da tilfelle 2 (`a = b`), altså `Θ(n log n)`:

- **Single-pivot**: `T(n) = 2T(n/2) + O(n)` → `Θ(n log n)`.
- **Dual-pivot**: `T(n) = 3T(n/3) + O(n)` → `Θ(n log n)`.

Verste tilfelle er teoretisk `O(n^2)` for begge ved svært ubalansert deling, men stokking (single-pivot) og fiks 1 (dual-pivot) gjør dette praktisk usannsynlig, mens 3-veis partisjonering / fiks 2 hindrer `O(n^2)` på mange like elementer.

## Tidsmålinger

Målt med `cargo run --release`, `n = 50 000 000` tall per tabell (i64), en kjøring
per algoritme/datasett (ikke repetisjoner, siden data er sortert etter første
kjøring). Alle sjekksum- og rekkefølgetester besto.

| Algoritme     | Datasett            | Tid       |
|---------------|----------------------|----------:|
| single-pivot  | tilfeldige tall      |  6 398 ms |
| dual-pivot    | tilfeldige tall      |  5 037 ms |
| single-pivot  | mange duplikater     |  8 416 ms |
| dual-pivot    | mange duplikater     |  4 665 ms |
| single-pivot  | sortert fra før      |  8 642 ms |
| dual-pivot    | sortert fra før      |    943 ms |
| single-pivot  | baklengs sortert     |  8 641 ms |
| dual-pivot    | baklengs sortert     |    963 ms |

## Konklusjon: hvilken variant er raskest?

**Dual-pivot vinner klart i alle fire tilfellene.**

- **Tilfeldige tall:** dual-pivot er ca. 20-25 % raskere, fordi tre deler per partisjonering gir færre rekursjonsnivåer enn to.
- **Mange duplikater:** dual-pivot er nesten dobbelt så rask, siden fiks 2 hopper over midtintervallet når pivotene er like.
- **Sortert / baklengs sortert:** dual-pivot er ca. 9x raskere enn på tilfeldige tall, siden fiks 1 gir nesten perfekt balanserte partisjoner på sorterte tabeller.
- **Single-pivot** viser derimot ingen forskjell mellom datasettene (alle rundt 6 400-8 600 ms), nettopp fordi stokkingen gjør ytelsen uavhengig av inndataens rekkefølge.

Dual-pivot (med de to fiksene) er altså raskest overalt, og spesielt overlegen på tabeller som allerede er helt eller delvis sortert.
