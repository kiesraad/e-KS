import type { AuthorisedPerson } from "./authorisedPerson";

export interface Candidate {
  initials: string;
  lastNamePrefix?: string;
  lastName: string;
  firstName?: string;
  gender?: string;
  dateOfBirth?: string;
  postalCode?: string;
  houseNumber?: string;
  houseNumberAddition?: string;
  streetName?: string;
  locality?: string;
  countryCode?: string;
  authorisedPerson?: AuthorisedPerson;
}
