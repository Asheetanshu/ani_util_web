function ham_menu(){
  const sidebar = document.getElementById("sidebar_section");
  sidebar.classList.toggle("sidebar_expanded");
  const home_svg = 
    `
  <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 0 24 24" width="24" focusable="false" aria-hidden="true" style="pointer-events: none; display: inherit; width: 100%; height: 100%;"><path d="m11.485 2.143-8 4.8-2 1.2a1 1 0 001.03 1.714L3 9.567V20a2 2 0 002 2h6v-7h2v7h6a2 2 0 002-2V9.567l.485.29a1 1 0 001.03-1.714l-2-1.2-8-4.8a1 1 0 00-1.03 0ZM5 8.366l7-4.2 7 4.2V20h-4v-5.5a1.5 1.5 0 00-1.5-1.5h-3A1.5 1.5 0 009 14.5V20H5V8.366Z"></path></svg>
  `
  const recent_svg =
  `
  <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 0 24 24" width="24" focusable="false" aria-hidden="true" style="pointer-events: none; display: inherit; width: 100%; height: 100%;"><path d="M8.76 1.487a11 11 0 11-7.54 12.706 1 1 0 011.96-.4 9 9 0 0014.254 5.38A9 9 0 0016.79 4.38 9 9 0 004.518 7H7a1 1 0 010 2H1V3a1 1 0 012 0v2.678a11 11 0 015.76-4.192ZM12 6a1 1 0 00-1 1v5.58l.504.288 3.5 2a1 1 0 10.992-1.736L13 11.42V7a1 1 0 00-1-1Z"></path></svg>
  `

}

async function ani_search(event){
  event.preventDefault();
  const srchq = document.getElementById("ani_search_input").value;
  if (srchq === ""){
    srchq.focus;
    console.log("I guess it is focused");
    return;

  }
  try {
    const response = await fetch("http://127.0.0.1:6769/app/ani_search",{
      method : "POST",
      headers : {
        "Content-type" : "application/json"
      },
      body: JSON.stringify(
        {
          querry : srchq,
        }
      )
    });
    const reply = await response.json();
    const imgsrc = document.getElementsByClassName("ani_banner_img")
    imgsrc.src = reply["image"];
    console.log(reply);
  }catch (error){
    console.log(`Error : ${error}`);

  }
}

const ham_but = document.getElementById("ham_menu_button");
const ani_srch = document.querySelector("form");
ani_srch.addEventListener("submit" , ani_search);
ham_but.onclick = ham_menu;
