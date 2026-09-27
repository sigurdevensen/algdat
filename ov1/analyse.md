# Oppgave 1-2

Programmet består av tre separate løkker som hver itererer over en array av lengde n.

1. Generering av prisendringer: en iterasjon over n elementer: O(n)
2. Beregning av faktiske priser: en iterasjon over n elementer: O(n)
3. Finn beste kjøps og salgstidspunkt: en iterasjon over n elementer: O(n)
   
Algoritmen holder løpende oversikt over laveste pris sett så langt og beregner profit for hver dag som potensiell salgsdag. Dersom profiten er bedre enn tidligere beste profit, så endres kjøp og salgstidspunktet. Dette gjøres i en enkel løkke som går gjennom alle prisene.

Siden løkkene kjører sekvensielt og ikke er nøstet, er total tidskompleksitet:

O(n) + O(n) + O(n) = O(n)

-> Algoritmen er lineær, hvis n dobles, dobles kjøretiden.

# Oppgave 1-3
Ved n=1000000 er kjøretiden ca 5-8ms
Ved n=2000000 er kjøretiden ca 16-19ms
Det vil si at kjøretiden faktisk dobles.
Samme mønster kan sees ved n=10000000:
Tid brukt: 85.7289ms
til n=20000000:
Tid brukt: 165.8161ms