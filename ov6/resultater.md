# Øving 6 - Resultater

LZ77 + Huffman med to trær (deflate-oppsett), utvidet med 1 MB vindu, optimal parsing og flere blokker.
Komprimering og dekomprimering kjøres hver for seg, og alle filene under ble pakket ut helt like originalen (`cmp`).

| Fil                                       |    Original | Komprimert | Andel av original | Første versjon |
|-------------------------------------------|------------:|-----------:|------------------:|---------------:|
| diverse.lyx                               |     184 300 |      9 652 |              5.2% |         10 863 |
| diverse.txt                               |      16 973 |      5 206 |             30.7% |          5 468 |
| Twenty_thousand_leagues_under_the_sea.txt |     609 116 |    199 425 |             32.7% |        231 164 |
| enwik6                                    |   1 000 000 |    306 164 |             30.6% |        355 457 |
| enwik7                                    |  10 000 000 |  3 031 154 |             30.3% |      3 682 224 |
| enwik8                                    | 100 000 000 | 30 038 879 |             30.0% |     36 430 664 |

"Første versjon" er den enkle varianten (32 kB vindu, grådig/lat matching, ett fast filhode).
Til sammenligning gir `gzip -9` 10 752 byte på diverse.lyx og ca. 36,4 MB på enwik8.

enwik8 tar ca. 60 s å komprimere og ca. 1 s å pakke ut.

## Forbedringer fra første versjon

1. **Større vindu og lengre treff:** 1 MB bakover (40 avstandskoder) og treff opptil 32 770 byte (56 lengdekoder), med samme mønster som deflate-tabellene.
2. **Optimal parsing:** i stedet for å ta lengste treff grådig, regnes kostnaden i bits for hvert valg ut fra huffmankodene, og billigste vei gjennom blokka velges (dynamisk programmering, to runder).
3. **Flere blokker:** hver blokk på 1 MB får egne huffmantrær, så kodene tilpasser seg innholdet.
4. **Komprimert blokkhode:** kodelengdene kjørelengdekodes og huffmankodes som i deflate, så hodet er ca. 60-90 byte i stedet for 158.

## Hvordan kjøre programmet

```terminal
cargo run --release --bin komprimering -- diverse.lyx
cargo run --release --bin dekomprimering -- diverse.lyx.komp
fc /b diverse.lyx diverse.lyx.dekomp
```

Se `instructions.md` for detaljer.
