const copyButtons = document.querySelectorAll("[data-copy]");

copyButtons.forEach((button) => {
  button.addEventListener("click", async () => {
    const targetId = button.getAttribute("data-copy");
    const target = document.getElementById(targetId);
    if (!target) {
      return;
    }

    try {
      await navigator.clipboard.writeText(target.innerText.trim());
      button.classList.add("copied");
      button.textContent = "Copied";
      setTimeout(() => {
        button.classList.remove("copied");
        button.textContent = "Copy";
      }, 1600);
    } catch (error) {
      button.textContent = "Failed";
      setTimeout(() => {
        button.textContent = "Copy";
      }, 1600);
    }
  });
});
