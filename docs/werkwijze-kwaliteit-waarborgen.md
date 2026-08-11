# Werkwijze e-KS team: kwaliteit waarborgen

Dit document beschrijft de werkwijze van het e-KS team en hoe er kwaliteit wordt gewaarborgd. Kwaliteit is hierbij geen aparte stap aan het einde van het ontwikkelproces, maar een geïntegreerd onderdeel van de werkwijze. 

Er is niet alleen aandacht voor de technische kwaliteit, maar ook of de gebouwde functionaliteit voldoet aan wetgeving en de behoeftes ondersteunt. 


## Ontwikkeling

### Algemene werkwijze
- Er wordt gezorgd voor continue kwaliteit door **iteratief te ontwikkelen**:
  - Het e-KS team werkt in twee wekelijkse sprints, inclusief alle sprintevents.  
  - Refinement wordt gedaan vlak voordat er wordt begonnen met de ontwikkeling van nieuwe functionaliteit
  - De Product Owner stemt zowel voor als na de ontwikkeling de functionaliteit af met interne stakeholders
  - Playwright tests worden gelijktijdig met de ontwikkeling geschreven
  - Na elke pull request worden de CI/CD pipeline, met quality gates, afgetrapt
- Er wordt gezorgd voor een gezamenlijk en objectief kwaliteitsniveau door de **Definition of Ready (DoR) en Definition of Done (DoD)**:
  - DoR: issue mag alleen worden opgepakt als er is voldaan aan de DoR [link]
  - DoD: issue is pas klaar als er is voldaan aan de DoD [link]
- Kwaliteit wordt verhoogd door het **vierogenprincipe**:
  - Elke pull request wordt gereviewed door een andere ontwikkelaar voordat code gemerged mag worden



  
### CI/CD pipeline
- Testing
  - Alle playwright tests moeten slagen
  - Alle unit testen moeten slagen
  - Er moet een unit test coverage van >80% zijn op nieuwe code
- Code style
  - Rust format geeft geen warning
  - Clippy geeft geen warnings
  - Geen Biome linting errors op CSS en TypeScript code
  - Geen djlint format warnings op HTML templates
  - Lekker meta: de CI/CD pipeline configuratie mag geen zizmor linting errors bevatten
  - Elke bestand in het archief moet eindigen met een lijntje met een een newline ('\n') karakter
- Algemene kwaliteit
  - Sigrid score van minimaal 3.5 ster op nieuwe code (dit is geen gate, de job faalt maar mergen mag nog steeds)
  - SonarQube quality gate moet slagen op nieuwe code (we gebruiken hiervoor de default "sonarway" configuratie)
- Dependency management
  - Er mogen geen Rust crates worden toegevoegd die niet aan onze kwaliteit/licensing standaarden voldoen (zie deny.toml voor exacte configuratie)
  - Cyclic dependencies op top level Rust modules zijn niet toegestaan (wordt afgedwongen door dylint)

## Testen

## Externe kwaliteitstoetsen

SIG

SonarCube

## Juiste functionaliteit

usecases 
Testen door PO
 

### interne stakeholders

### externe stakeholders

## Documentatie
