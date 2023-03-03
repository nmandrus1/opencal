// --------------------------------------------------------
//              collapsible tag groups
window.onload = function() {
    Array.from(document.getElementsByClassName("collapsible")).forEach(button => {
        button.addEventListener("click", function() {
            this.classList.toggle("active");
            var content = this.nextElementSibling;
            if (content.style.maxHeight){
              content.style.maxHeight = null;
            } else {
              content.style.maxHeight = content.scrollHeight + "px";
            }
          });
    });
}

// --------------------------------------------------------