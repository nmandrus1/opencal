

window.onload = function() {
  // used for collapsible tag groups
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
      button.click();
  });

  // used to populate the calendar
  var output = "";
  var days = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
  var hours = ["12am", "1am", "2am", "3am", "4am", "5am", "6am", "7am", "8am", "9am", "10am", "11am", 
                "12pm", "1pm", "2pm", "3pm", "4pm", "5pm", "6pm", "7pm", "8pm", "9pm", "10pm", "11pm"];
  hours.forEach(function(hour) {
    output += "<tr id=\"" + hour + "-row\">";
    output += "<td id=\"" + hour + "\">" + hour + "</td>";
    days.forEach(function(day) {
      output += "<td id=\"" + hour + "-" + day + "\"></td>";
    });
    output += "</tr>";
  });
  $("#calendarTable tbody").after(output);
}