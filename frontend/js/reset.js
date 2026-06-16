function is_valid_uname(str) {
  return /^[a-zA-Z0-9]{8,16}$/.test(str); 
}

function is_vaild_email(str){
  return /^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$/.test(str);
}

async function check_input(event){
  event.preventDefault();
  const umail = document.getElementById('uname_or_email').value;
  const lgst = document.getElementById('login_status');

  if (umail === ""){
    lgst.style.color = "var(--wine-plum)";
    lgst.innerText = "Error: Please fill out username or email.";
    return;
    
  }else if (umail.includes("@")){
    if (!is_vaild_email(umail)){
      lgst.style.color = "var(--wine-plum)";
      lgst.innerText = `Error: Not a valid Email`;
    } 
  } else if (!is_valid_uname(umail)){
    lgst.style.color = "var(--wine-plum)";
    lgst.innerText = `Error: Username should only contain\n(a to z\t,A to Z\t,0 to 9)\n{char limit : 16}`;
    return;
  }else {
    console.log(`Username:"${umail}"\n`)
    try{
      const response = await fetch("http://127.0.0.1:6769/app/reset" , {
        method : "POST",
        headers : {
          "Content-type" : "application/json"
        },
        body: JSON.stringify(
          {
            umail : umail,
          }
        )
      });

      const reply = await response.text();
      if (reply === "1"){                                         // username or email found
        lgst.style.color = "var(--green)";
        lgst.innerHTML = `You exist in my system.\nCheck Your mail to reset password\nAfter that <a href="/pages/forgot">Reset Passord</a>`;
        // after finding out that they exist we mail them an otp and
        // a reset password link which will have a time limit of lets
        // say 15 min and then they will have to reset their password 
        // also enter the otp in that specified time in the forgot password 
        // page or if possible we can extend this page only with three new 
        // input fields one with otp and 2 for passwd and cnf passwd

      }else if (reply === "2"){                                   // did not match any uname or email
        lgst.style.color = "var(--pearl-beige)";
        lgst.innerHTML = 
        `Did not find that account Try again or <a href="/signup"  
        style = "color : var(--pearl-beige); text-decoration : underline;">Sign UP</a>`;
      }else {
        lgst.style.color = "var(--wine-plum)";
        lgst.innerText = `Error : "${reply}"`;
      }
    }catch(network_error){
      lgst.style.color = "var(--wine-plum)";
      lgst.innerText = `Network error "${network_error}"`;
    }
  }
}


const lform = document.querySelector("form");
lform.addEventListener("submit" , check_input);
const umail_input = document.getElementById('uname_or_email');
const lgst_input = document.getElementById('login_status');
umail_input.addEventListener("input" , function(){
  lgst_input.innerHTML = `Don't have an Account? <a href ="/signup">Sign Up</a>`;
});

