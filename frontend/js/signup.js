function is_valid_uname(str) {
  return /^[a-zA-Z0-9]{8,16}$/.test(str); 
}
function is_valid_passwd(str){
  return /^[a-zA-Z0-9!@#$%^&*]{8,32}$/.test(str);
}

function reveal_fun(event){
  event.preventDefault();
  const passwd = document.getElementById("passwd");
  const svg = document.getElementById("eye_svg");

  const eye_open = `
<path d="M12 5C8.24261 5 5.43602 7.4404 3.76737 9.43934C2.51521 10.9394 2.51521 13.0606 3.76737 14.5607C5.43602 16.5596 8.24261 19 12 19C15.7574 19 18.564 16.5596 20.2326 14.5607C21.4848 13.0606 21.4848 10.9394 20.2326 9.43934C18.564 7.4404 15.7574 5 12 5Z" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
<path d="M12 15C13.6569 15 15 13.6569 15 12C15 10.3431 13.6569 9 12 9C10.3431 9 9 10.3431 9 12C9 13.6569 10.3431 15 12 15Z" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
  `;
  const eye_close = `
    <path d="M4.71 3.29a1 1 0 0 0-1.42 1.42l5.63 5.63a3.5 3.5 0 0 0 4.74 4.74l5.63 5.63a1 1 0 0 0 1.42 0 1 1 0 0 0 0-1.42zM12 13.5a1.5 1.5 0 0 1-1.5-1.5v-.07l1.56 1.56z" fill="currentColor"/>
      <path d="M12.22 17c-4.3.1-7.12-3.59-8-5a13.7 13.7 0 0 1 2.24-2.72L5 7.87a15.89 15.89 0 0 0-2.87 3.63 1 1 0 0 0 0 1c.63 1.09 4 6.5 9.89 6.5h.25a9.48 9.48 0 0 0 3.23-.67l-1.58-1.58a7.74 7.74 0 0 1-1.7.25z" fill="currentColor"/>
      <path d="M21.87 11.5c-.64-1.11-4.17-6.68-10.14-6.5a9.48 9.48 0 0 0-3.23.67l1.58 1.58a7.74 7.74 0 0 1 1.7-.25c4.29-.11 7.11 3.59 8 5a13.7 13.7 0 0 1-2.29 2.72L19 16.13a15.89 15.89 0 0 0 2.91-3.63 1 1 0 0 0-.04-1z" fill="currentColor"/>
  `;

  if (passwd.type == "password"){
    passwd.type = "text";
    svg.innerHTML = eye_open;
  }else {
    passwd.type = "password";
    svg.innerHTML = eye_close;
  }
}

async function check_input(event){
  event.preventDefault();
  const uname = document.getElementById('uname').value;
  const ogpasswd = document.getElementById('passwd_og').value;
  const passwd = document.getElementById('passwd').value;
  const email = document.getElementById('email').value;
  const lgst = document.getElementById('login_status');


  if (uname === "" || passwd === "" || email === "" || ogpasswd === ""){
    lgst.style.color = "var(--wine-plum)";
    lgst.innerText = "Error: Please fill out all the feilds";
    return;
  }else if (!is_valid_uname(uname)){
    lgst.style.color = "var(--wine-plum)";
    lgst.innerText = `Error: Username should only contain\n(a to z\t,A to Z\t,0 to 9)\n{char limit : 16}`;
    return;
  }else if (!is_valid_passwd(ogpasswd)){
    lgst.style.color = "var(--wine-plum)";
    lgst.innerText = `Error: Password should only contain\n(a to z\t,A to Z\t,0 to 9\tOr Special charecters "!@#$%^&*")\n{char limit : 32}`;
    return;
  }else if (!is_valid_passwd(passwd)){
    lgst.style.color = "var(--wine-plum)";
    lgst.innerText = `Error: Confirm Password should only contain\n(a to z\t,A to Z\t,0 to 9\tOr Special charecters "!@#$%^&*")\n{char limit : 32}`;
    return;
  }else if (ogpasswd !== passwd){
    lgst.style.color = "var(--pearl-beige)";
    lgst.innerText = `Error: Password and Confirm Password Must Be same`;
    return;
  }else {
    console.log(`Username:"${uname}"\nPassword:"${passwd}"\nEmail:"${email}"\n`)
    try{
      const response = await fetch("http://127.0.0.1:6769/app/signup" , {
        method : "POST",
        headers : {
          "Content-type" : "application/json"
        },
        body: JSON.stringify(
          {
            uname : uname,
            passwd : passwd,
            email : email,
          }
        )
      });

      const reply = await response.text();
      if (reply === "1"){                                         // username & email valid 
        lgst.style.color = "var(--green)";
        lgst.innerHTML = `SignUp successfull.<a href="/login" style ="color : var(--pearl-beige); text-decoration : underline;">Login now</a>`;
      }else if (reply === "2"){                                   // username not unique
        lgst.style.color = "var(--pearl-beige)";
        lgst.innerHTML = `Username not unique . Try again or <a href="/login" style ="color : var(--pearl-beige); text-decoration : underline;">Login now ?</a>`;
      }else if (reply === "3"){                                   // email not unique
        lgst.style.color = "var(--wine-plum)";
        lgst.innerHTML = `Email not unique. User already exists <a href="/login" style ="color : var(--pearl-beige); text-decoration : underline;">Login now ?</a>`;
      }else{
        lgst.style.color = "var(--wine-plum)";
        lgst.innerText = `Internal Error : "${reply}"`;
      }
    }catch(network_error){
      lgst.style.color = "var(--wine-plum)";
      lgst.innerText = `Network error "${network_error}"`;
    }
    return;
  }
}


const lform = document.querySelector("form");
lform.addEventListener("submit" , check_input);
const uname_input = document.getElementById('uname');
const passwd_input = document.getElementById('passwd');
const email_input = document.getElementById('email');
const lgst_input = document.getElementById('login_status');
const ogpasswd_input = document.getElementById('passwd_og');
uname_input.addEventListener("input" , function(){
  lgst_input.innerHTML = `Already have an Account ? <a href="/login">Login</a>`
});
passwd_input.addEventListener("input" , function(){
  lgst_input.innerHTML = `Already have an Account ? <a href="/login">Login</a>`
});
email_input.addEventListener("input" , function(){
  lgst_input.innerHTML = `Already have an Account ? <a href="/login">Login</a>`
});
ogpasswd_input.addEventListener("input" , function(){
  lgst_input.innerHTML = `Already have an Account ? <a href="/login">Login</a>`
});
const reveal_but = document.getElementById("reveal");
reveal_but.onclick=reveal_fun;
