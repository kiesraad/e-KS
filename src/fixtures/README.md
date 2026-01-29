# Example persons

The list of example persons is taken from https://github.com/BRP-API

See https://github.com/BRP-API/Haal-Centraal-BRP-bevragen/tree/master/test-data/personen-mock

## Generating the CSV

```sh
#!/bin/bash
fields=$(jq -r '.[] | [
    .burgerservicenummer,
    .geslacht.code,
    .naam.voornamen,
    .naam.geslachtsnaam,
    .geboorte.datum,
    .verblijfplaats.straat,
    .verblijfplaats.huisnummer,
    .verblijfplaats.postcode,
    .verblijfplaats.woonplaats
] | @csv' test-data.json)

echo "burgerservicenummer,geslacht,voornamen,geslachtsnaam,geboortedatum,straat,huisnummer,postcode,woonplaats" > output.csv
echo "$fields" >> output.csv
```
