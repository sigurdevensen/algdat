# Øving 5 - Resultater

## Maks flyt for hver graf

| Graf       | Kilde | Sluk | Maks flyt | Antall flytøkende veier |
|------------|:-----:|:----:|----------:|------------------------:|
| flytgraf1  |   0   |  7   | 10        | 8                       |
| flytgraf2  |   0   |  1   | 27        | 4                       |
| flytgraf3  |   0   |  1   | 42        | 6                       |
| flytgraf4  |   0   |  7   | 11        | 9                       |
| flytgraf5  |   0   |  7   | 90        | 5                       |

flytgraf1 og flytgraf4 stemmer med fasiten i oppgaveteksten (10 og 11). flytgraf3 og flytgraf5 var de to grafene vi ikke fikk oppgitt fasit for og de endte på 42 og 90.

Slik ser output ut, her med flytgraf5 som eksempel:

```c
Maksimum flyt fra 0 til 7 med Edmond-Karp
Økning : Flytøkende vei
    10 0 2 3 7
    25 0 2 4 7
    10 0 5 6 7
    40 0 5 6 1 7
     5 0 2 4 6 1 7
Maksimal flyt ble 90
```

## Hvordan kjøre programmet

```terminal
cargo run
```
