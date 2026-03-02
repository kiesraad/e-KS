import highlightActiveLinks from "./scripts/highlights-alerts/active-link";
import alertSuccess from "./scripts/highlights-alerts/alert-success";
import bsnInput from "./scripts/form-inputs/bsn-input";
import setupClickRow from "./scripts/table-interaction/click-row";
import countryCodeInput from "./scripts/form-inputs/country-input";
import dateInput from "./scripts/form-inputs/date-input";
import setupDirtyForms from "./scripts/form-inputs/dirty-form";
import highlightRow from "./scripts/highlights-alerts/highlight-row";
import initialsInput from "./scripts/form-inputs/initials-input";
import localitySuggestions from "./scripts/form-inputs/locality-suggestions";
import addressLookup from "./scripts/form-inputs/lookup";
import setupModal from "./scripts/generic-ui/modal";
import setupOverlay from "./scripts/generic-ui/overlay";
import setupPositionPreview from "./scripts/form-inputs/position-preview";
import setupTextSearch from "./scripts/generic-ui/search";
import setupSelectAllCheckbox from "./scripts/form-inputs/select-all-checkbox";
import setupSortable from "./scripts/generic-ui/sortable";
import setupStickyNav from "./scripts/generic-ui/sticky-nav";

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
