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
De CI/CD pipeline bevat de volgende quality gates
- Testing
  - Alle playwright tests moeten slagen
  - Alle unit testen moeten slagen
  - Er moet een unit test coverage van >80% zijn op nieuwe code
- Code style
  - Rust format geeft geen warning
  - Clippy geeft geen warnings
  - Geen Biome linting errors op CSS en TypeScript code
  - Geen djlint format warnings op HTML templates
  - De CI/CD pipeline configuratie mag geen zizmor linting errors bevatten
  - Elke bestand in het archief moet eindigen met een lijntje met een een newline ('\n') karakter
- Algemene kwaliteit
  - Sigrid score van minimaal 3.5 ster op nieuwe code (dit is geen gate, de job faalt maar mergen mag nog steeds)
  - SonarQube quality gate moet slagen op nieuwe code (we gebruiken hiervoor de default "sonarway" configuratie)
- Dependency management
  - Er mogen geen Rust crates worden toegevoegd die niet aan onze kwaliteit/licensing standaarden voldoen (zie deny.toml voor exacte configuratie)
  - Cyclic dependencies op top level Rust modules zijn niet toegestaan (wordt afgedwongen door dylint)

## Testen

Verder uitwerken, maar alvast wat punten:
- unit testen
- playwright testen
- Testen hangen aan usecases
- Elk issue wordt functioneel getest door de Product Owner
- Testen met eindgebruikers (frequentie: meerdere keren per jaar)

## Externe kwaliteitstoetsen

SIG

SonarCube

## Juiste functionaliteit

Om tot de juist functionaliteit te komen werkt het e-KS team met use cases (gebruiksscenario's) deze zijn afgeleid van de Kieswet. Vanuit deze use cases worden de epics gemaakt en  

### interne stakeholders
Frequentie: wekelijks  
Wie: afdelingen "Juridische Kennis en Advies" en "Regie Kwaliteit Verkiezingsketen"  
Doel: Vooraf en achteraf feedback ophalen  
Hoe:   
- Refinement van de komende functionaliteit
- Review (demo) van gebouwde functionaliteit
- Testen van functionaliteit op functioneel niveau
- Testen van functionaliteit op juridische kaders 

### externe stakeholders
Frequentie: meerdere keren per jaar
Wie: Zowel politieke partijen als gemeentes en waterschappen
Doel: Vooraf en achteraf feedback ophalen  
Hoe:   
- Review (demo) van gebouwde functionaliteit
- Testen van functionaliteit op functioneel niveau

## Documentatie
