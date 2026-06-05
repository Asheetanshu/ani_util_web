function ham_menu(){
  const sidebar = document.getElementById("sidebar_wrapper");
  sidebar.style.boxShadow = "0px 0px 10px 1px var(--wine-plum)";
  sidebar.style.background = "var(--carbon-black)";
  sidebar.classList.toggle("sidebar_hidden");
}
const ham_but = document.getElementById("ham_menu_button");
ham_but.onclick = ham_menu;
