# Werkwijze e-KS team: kwaliteit waarborgen

Dit document beschrijft de werkwijze van het e-KS team en hoe er kwaliteit wordt gewaarborgd. Kwaliteit is hierbij geen aparte stap aan het einde van het ontwikkelproces, maar een geïntegreerd onderdeel van de werkwijze. 

Er is aandacht voor de technische kwaliteit en functionele kwaliteit. Functionele kwaliteit wordt gewaarborgd door de applicatie te toetsen aan wetgeving en gebruikersbehoeftes.

De belangrijkste kwaliteitsattributen voor de ontwikkeling van e-KS zijn:
- Betrouwbaarheid: Kan je de software vertrouwen?
- Bruikbaarheid: Is de software makkelijk te gebruiken (voor alle bedoelde gebruikers)?
- Beveiliging: Biedt de software voldoende bescherming tegen ongewenst gebruik?
- Testbaarheid: Is het eenvoudig om de software te testen en controleren?
- Onderhoudbaarheid: Kan de software eenvoudig worden onderhouden en aangepast?
- Beheerbaarheid: Is de software eenvoudig te installeren, ondersteunen en onderhouden?

## Ontwikkeling

### Algemene werkwijze
- Er wordt gezorgd voor continue kwaliteit door **iteratief te ontwikkelen**:
  - Het e-KS werkt [agile](https://agilemanifesto.org/) door middel van [Scrum](https://www.scrum.org/)
  - Het e-KS team werkt in twee wekelijkse sprints, inclusief alle Scrum ceremonies.
  - Refinement wordt deels gedaan tijdens de sprint planning en deels ad hoc voordat er wordt begonnen met de ontwikkeling van nieuwe functionaliteit
  - De Product Owner stemt zowel voor als na de ontwikkeling de functionaliteit af met interne stakeholders
  - Systeem testen worden gelijktijdig met de ontwikkeling geïmplementeerd met [Playwright](https://playwright.dev/)
  - Nieuwe features worden ingediend door middel van Pull Requests
  - Pull Requests worden pas geaccepteerd in de main branch zodra:
     - De CI/CD pipeline, met quality gates, slaagt
     - De wijziging door minstens één andere ontwikkelaar is gereviewd en goedgekeurd (vierogenprincipe)
- Er wordt gezorgd voor een gezamenlijk en objectief kwaliteitsniveau door de **Definition of Ready (DoR) en Definition of Done (DoD)**:
  - DoR: issue mag alleen worden opgepakt als er is voldaan aan de [DoR](docs/definition-of-ready.md)
  - DoD: issue is pas klaar als er is voldaan aan de [DoD](docs/definition-of-done.md)
 
### Continuous Integration / Continuous Delivery (CI/CD)

CI/CD  helpt om risico’s  te mitigeren. Door codewijzigingen regelmatig (bij elke PR) te integreren, testen en opleveren, worden fouten eerder ontdekt en wordt de kans op problemen bij het mergen en opleveren van nieuwe code verkleind. Door een CI/CD-pipeline in te richten kunnen deze stappen geautomatiseerd worden gedaan.

De CI/CD pipeline binnen e-KS bevat de volgende quality gates:
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
  - Elke bestand in het archief moet eindigen met een witregel
- Algemene kwaliteit
  - Sigrid score van minimaal 3.5 ster op nieuwe code (dit is geen gate, de job faalt maar mergen mag nog steeds)
  - SonarQube quality gate moet slagen op nieuwe code (we gebruiken hiervoor de default "sonarway" configuratie)
- Dependency management
  - Er mogen geen Rust crates worden toegevoegd die niet aan onze kwaliteit/licensing standaarden voldoen (zie deny.toml voor exacte configuratie)
  - Cyclic dependencies op top level Rust modules zijn niet toegestaan (wordt afgedwongen door dylint)

## Juiste functionaliteit

Om tot de juist functionaliteit te komen werkt het e-KS team met use cases (gebruiksscenario's) deze zijn afgeleid van de Kieswet. Vanuit deze use cases worden de epics gemaakt. Vanuit de epics worden issues aangemaakt die door de ontwikkelaars worden opgepakt. Vervolgens worden de issues getest door de Product Owner. Daarnaast wordt de applicatie getest door interne en externe stakeholders.

### Interne stakeholders
Frequentie: wekelijks  
Wie: afdelingen "Juridische Kennis en Advies" en "Regie Kwaliteit Verkiezingsketen"  
Doel: Vooraf en achteraf feedback ophalen  
Hoe:   
- Refinement van de komende functionaliteit
- Review (demo) van gebouwde functionaliteit
- Testen van functionaliteit op functioneel niveau
- Testen van functionaliteit op juridische kaders 

### Externe stakeholders
Frequentie: meerdere keren per jaar  
Wie: Zowel politieke partijen als gemeentes en waterschappen  
Doel: Vooraf en achteraf feedback ophalen  
Hoe:   
- Review (demo) van gebouwde functionaliteit
- Testen van functionaliteit op functioneel niveau

## Testen

De volgende tests worden uitgevoerd om de kwaliteit van de software te borgen:

- Rust unit testen: Testen onderdelen van de back-end. Alle code wordt afgedekt met unit tests.
- Playwright testen: Testen de front-end. Twee soorten testen:
  - 1) Testen gekoppeld aan use cases
  - 2) End-to-end tests vanuit het gebruikersperspectief, op epic-niveau.
- Functioneel testen door de Product Owner (PO): elk issue wordt getest door de PO om te kijken of de gewenste functionaliteit is ontwikkeld.
- Testen met eindgebruikers: Meerdere keren per jaar wordt er getest met eindgebruikers. Er wordt dan gekeken of de software voldoet aan hun behoeftes.
- Exploratief testen: Vanuit gebruikersperspectief testen we per issue of er bugs of verbeterpunten zijn.

## Externe kwaliteitstoetsen

SIG en SonarQube worden ingezet voor geautomatiseerde analyse en borging van de (technische) kwaliteit van de code. Ze analyseren de code onder andere op onderhoudbaarheid, architectuur, beveiliging, voorspelbaarheid. SIG en SonarQube zijn geïntegreerd in de CI/CD pipeline.





