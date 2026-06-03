import bsnInput from "./scripts/form-inputs/bsn-input";
import countryCodeInput from "./scripts/form-inputs/country-input";
import dateInput from "./scripts/form-inputs/date-input";
import electionRegion from "./scripts/form-inputs/election-region";
import listDesignation from "./scripts/form-inputs/list-designation";
import setupFileImport from "./scripts/form-inputs/file-import";
import initialsInput from "./scripts/form-inputs/initials-input";
import localitySuggestions from "./scripts/form-inputs/locality-suggestions";
import addressLookup from "./scripts/form-inputs/lookup";
import setupPositionPreview from "./scripts/form-inputs/position-preview";
import setupSelectAllCheckbox from "./scripts/form-inputs/select-all-checkbox";
import setupModal from "./scripts/generic-ui/modal";
import setupOverlay from "./scripts/generic-ui/overlay";
import setupRememberScroll from "./scripts/generic-ui/remember-scroll";
import setupTextSearch from "./scripts/generic-ui/search";
import setupStepNav from "./scripts/generic-ui/step-nav";
import setupStickyNav from "./scripts/generic-ui/sticky-nav";
import highlightActiveLinks from "./scripts/highlights-alerts/active-link";
import alertSuccess from "./scripts/highlights-alerts/alert-success";
import highlightRow from "./scripts/highlights-alerts/highlight-row";
import setupClickCard from "./scripts/generic-ui/click-card";
import setupClickRow from "./scripts/table-interaction/click-row";
import setupSortable from "./scripts/table-interaction/sortable";

import "./styles/index.css";

// table interaction
setupClickRow();
setupClickCard();

// highlights and alerts
highlightActiveLinks();
alertSuccess();
highlightRow();

// form inputs
bsnInput();
countryCodeInput();
dateInput();
initialsInput();
setupFileImport();
addressLookup();
localitySuggestions();
setupPositionPreview();
setupSelectAllCheckbox();
electionRegion();
listDesignation();

// generic UI
setupStickyNav();
setupModal();
setupOverlay();
setupStepNav();
setupTextSearch();
setupSortable();
setupRememberScroll();
