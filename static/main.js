const searchbar = document.getElementById("searchbar");
const announcements = document.getElementById("announcements");

const map = L.map('map').setView([51.505, -0.09], 13);
const popup = L.popup();

const tiles = L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  maxZoom: 19,
  attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
}).addTo(map);

// The list of all the marker we can see on the map.
// We took care of inserting the ID of the lair associated with each marker
// to get it back easily when someone click on a marker.
var all_markers = []

// Called when someone click on a popup.
async function popup_opened(event) {
  console.log(event);
  open_popup(event.sourceTarget)

}

// Called when a popup is opened and must be called with the marker associated to the click.
async function open_popup(marker) {
  // First, get the ID associated with this marker.
  let id = marker.announce_id;

  // Second get the document associated with the id with all its content.
  let result = await fetch(`http://127.0.0.1:5000/lair/${id}`, {
    method: "GET"
  });
  let lair = await result.json();
  console.log(lair);

  // Third, create a popup with all the info + a button to delete the document.
  marker.bindPopup(`
  <div class="announce_popup">
    <div>
      <img src=${lair.image}/>
      <ul class="view_annonce">
        <li>${lair.title}</li>
        <li>${lair.description}</li>
      </ul>
    </div>
    <input type="button" value="Supprimer l'annonce"/>
  </div>
`).openPopup();

}

// This function updates the left menu and the map with the current
// position on the map and the current search.
async function refresh_announce(bounds) {
  // First thing first: We need to get the latest freshest info from the
  // backend, so we're going to compose a request containing the current
  // view of the map and the search currently going on if there is one.

  // tl.lat = ne.lat
  // tl.lng = sw.lng
  // br.lat = sw.lat
  // br.lng = ne.lng
  let tl_lat = bounds._northEast.lat;
  let tl_lng = bounds._southWest.lng;
  let br_lat = bounds._southWest.lat;
  let br_lng = bounds._northEast.lng;

  let query = `http://127.0.0.1:5000/lair?tl_lat=${tl_lat}&tl_lng=${tl_lng}&br_lat=${br_lat}&br_lng=${br_lng}`;
  if (searchbar.value != "") {
    let search = searchbar.value;
    query += `&search=${search}`;
  }

  let results = await fetch(query, {
    method: "GET"
  });
  let ret = await results.json();

  console.log(ret);
  // Now that we have the results we can update the website:
  // 1. We should clear the left menu.
  // 2. We should clear all the marker on the map.
  // 3. We should re-insert both the marker and the item in the menu.

  // 1.
  announcements.innerHTML = "";

  // 2.
  for (var i = 0; i < all_markers.length; i++) {
    map.removeLayer(all_markers[i]);
  }
  all_markers = [];
    
  // 3.
  for (var i = 0; i < ret.length; i++) {
    let announce = ret[i];
    // Insert the new marker.
    let marker = L.marker([announce.lat, announce.lon], { title: announce.title } ).addTo(map)
    marker.on('click', popup_opened);
    marker.announce_id = announce.id;
    all_markers.push(marker);

    // Push the new element into to the list.
    announcements.innerHTML += `
    <div class="announce">
        <img src="${announce.image}"> 
        <span>${announce.title}</span>
    </div>`;
    
  }
}

function update_points(e) {
  let bounds = map.getBounds();
  refresh_announce(bounds);  
}

map.on('moveend', update_points);
map.on('zoomend', update_points);
searchbar.addEventListener('input', update_points);


function onMapClick(e) {
  console.log(e);

  let lat = e.latlng.lat;
  let lng = e.latlng.lng;
  popup
    .setLatLng(e.latlng)
    .setContent(`
<form action="" class="create_announce" onsubmit="insertDocument();return false;">
  <div class="form-example">
    <label for="titre">Titre de l'annonce</label>
    <input type="text" id="title" required />
  </div>
  <div class="form-example">
    <label for="description">Description</label>
    <input type="text" id="description" required />
  </div>
  <div class="form-example">
    <label for="image">URL de l'image</label>
    <input type="text" id="image" required />
  </div>
  <div class="form-example" style="display: none">
    <label for="lat">lat</label>
    <input type="text" id="lat" value="${lat}"/>
  </div>
  <div class="form-example" style="display: none">
    <label for="lng">lng</label>
    <input type="text" id="lng" value="${lng}"/>
  </div>
  <div class="form-example">
    <input type="submit" value="Créer" />
  </div>
</form>
`)
    .openOn(map);
}

async function insertDocument() {
  let title = document.getElementById("title").value;
  let description = document.getElementById("description").value;
  let image = document.getElementById("image").value;
  let lat = document.getElementById("lat").value;
  let lng = document.getElementById("lng").value;
  let cookie = document.cookie.split(";");
  for(i = 0; i < cookie.length; i++) {
    if (cookie[i].startsWith('cookie=')){
      cookie = cookie[i].substring('cookie='.length)
    }
  }

  let payload = {
    title: title,
    description: description,
    image: image,
    lat: Number(lat),
    lon: Number(lng),
  };
  console.log("Sending payload", payload); 
  console.log(cookie);
   
  let response = await fetch("http://127.0.0.1:5000/lair", {
    method: "POST",
    body: JSON.stringify(payload),
    headers: {
      "Content-Type": "application/json",
      "Authorization" : "Bearer " + cookie
    },
  });
  let ret = await response.json();
  // TODO: Handle the error
  // console.log(ret);  
  map.closePopup();
}

map.on('click', onMapClick);


refresh_announce(map.getBounds())

function openForm() {
  document.getElementById("register").style.display = "block";
}
function closeForm() {
  document.getElementById("register").style.display = "none";
}

async function create_Account() {
    let fullNameValue = document.getElementById("fname").value;
    let passwordValue = document.getElementById("password").value;

    // Send form info to SQL table for new users.
    let creationAccount = await fetch(`http://127.0.0.1:5000/user`, {
      method: "POST",
      body: JSON.stringify({fullName : fullNameValue, password : passwordValue}),
      headers: {
        "Content-Type": "application/json",
      }
    });
    let user = await creationAccount.json();
    console.log(user);
    document.cookie = "cookie="+ user["cookie"] + "; SameSite=None; Secure; path=/";
    window.location.href = "/static/index.html";
  }

async function login_account() {
  let fullNameValue = document.getElementById("fname").value;
  let passwordValue = document.getElementById("password").value

    // Send form info to SQL table for login.
    let login_account = await fetch(`http://127.0.0.1:5000/user/login`, {
      method: "POST",
      body: JSON.stringify({fullName : fullNameValue, password : passwordValue}),
      headers: {
        "Content-Type": "application/json",
      }
    });
    let user = await login_account.json();
    console.log(user);
    document.cookie ="cookie="+ user["cookie"] + "; SameSite=None; Secure; path=/";
}

