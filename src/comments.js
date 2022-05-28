var button = document.querySelector('#button');
button.remove();
var holder = document.querySelector('#holder');
var id = window.location.pathname.split('/').pop();
var url = `/comments/${id}`
var request = new XMLHttpRequest();
request.open('GET', url, true);
request.onload = function() {
    if (this.status == 200) {
        holder.innerHTML = this.responseText;
    }
}
request.send(null);
