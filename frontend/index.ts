import highlightActiveLinks from "./scripts/active-link";
import alertSuccess from "./scripts/alert-success";
import bsnInput from "./scripts/bsn-input";
import setupClickRow from "./scripts/click-row";
import countryCodeInput from "./scripts/country-input";
import dateInput from "./scripts/date-input";
import setupDirtyForms from "./scripts/dirty-form";
import highlightRow from "./scripts/highlight-row";
import initialsInput from "./scripts/initials-input";
import localitySuggestions from "./scripts/locality-suggestions";
import addressLookup from "./scripts/lookup";
import setupModal from "./scripts/modal";
import setupOverlay from "./scripts/overlay";
import setupPositionPreview from "./scripts/position-preview";
import setupTextSearch from "./scripts/search";
import setupSelectAllCheckbox from "./scripts/select-all-checkbox";
import setupSortable from "./scripts/sortable";
import setupStickyNav from "./scripts/sticky-nav";

import "./styles/index.css";

// table interaction
setupClickRow();

// highlights and alerts
highlightActiveLinks();
alertSuccess();
highlightRow();

// form inputs
bsnInput();
countryCodeInput();
dateInput();
initialsInput();
addressLookup();
localitySuggestions();
setupPositionPreview();
setupSelectAllCheckbox();
setupDirtyForms();

// generic UI
setupStickyNav();
setupModal();
setupOverlay();
setupTextSearch();
setupSortable();
