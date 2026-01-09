window.addEventListener("load", () => {
  document
    .querySelectorAll("div.select-all-checkbox > input[type='checkbox']")
    .forEach((element) => {
      const selectAllCheckbox = element as HTMLInputElement;
      const listId = selectAllCheckbox.getAttribute("for-checklist");
      const checkList: NodeListOf<HTMLInputElement> = document.querySelectorAll(
        `div[id=${listId}] > div.checkbox > input[type=checkbox]`,
      );

      // determine initial state onload
      determine_select_all_state(selectAllCheckbox, checkList);

      // add event listener for the select all checkbox
      selectAllCheckbox.addEventListener("change", (_) => {
        checkList.forEach((checkbox) => {
          checkbox.checked = selectAllCheckbox.checked;
        });
      });

      // add event listeners for all checkboxes in the checklist to update the select-all checkbox
      checkList.forEach((checkbox) => {
        checkbox.addEventListener("change", (_) => {
          determine_select_all_state(selectAllCheckbox, checkList);
        });
      });
    });
});

const determine_select_all_state = (
  selectAllCheckbox: HTMLInputElement,
  checkList: NodeListOf<HTMLInputElement>,
) => {
  // FIXME: indeterminate state doesn't render yet (it is however correctly set)
  selectAllCheckbox.indeterminate = false;
  if (Array.from(checkList).every((cb) => cb.checked)) {
    selectAllCheckbox.checked = true;
  } else if (Array.from(checkList).every((cb) => !cb.checked)) {
    selectAllCheckbox.checked = false;
  } else {
    // some are checked, some aren't => indeterminate
    selectAllCheckbox.indeterminate = true;
  }
};
