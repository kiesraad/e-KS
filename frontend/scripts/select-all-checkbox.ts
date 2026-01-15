window.addEventListener("load", () => {
  document
    .querySelectorAll(".select-all-checkbox > input")
    .forEach((element) => {
      const selectAllCheckbox = element as HTMLInputElement;
      const listId = selectAllCheckbox.getAttribute("for-checklist");
      const checkList: NodeListOf<HTMLInputElement> = document.querySelectorAll(
        `#${listId} input[type=checkbox]`,
      );

      // determine initial state onload
      determine_select_all_state(selectAllCheckbox, checkList);

      // add event listener for the select all checkbox
      selectAllCheckbox.addEventListener("change", (_) => {
        checkList.forEach((checkbox) => {
          checkbox.checked = selectAllCheckbox.checked;
        });
        determine_select_all_state(selectAllCheckbox, checkList);
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
  if (Array.from(checkList).every((cb) => cb.checked)) {
    selectAllCheckbox.checked = true;
    selectAllCheckbox.indeterminate = false;
  } else if (Array.from(checkList).every((cb) => !cb.checked)) {
    selectAllCheckbox.checked = false;
    selectAllCheckbox.indeterminate = false;
  } else {
    selectAllCheckbox.checked = false;
    selectAllCheckbox.indeterminate = true;
  }

  selectAllCheckbox.classList.toggle(
    "indeterminate",
    selectAllCheckbox.indeterminate,
  );
};
