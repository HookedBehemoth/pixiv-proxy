function inject(element, after = true) {
    let holder = element.parentNode
    let url = element.attributes.endpoint.value
    var request = new XMLHttpRequest()
    request.open('GET', url, true)
    request.onload = function() {
        if (this.status == 200) {
            if (after) {
                holder.innerHTML += this.responseText
            } else {
                holder.innerHTML = this.responseText + holder.innerHTML
            }
        }
    }
    request.send(null)
    element.remove()
}
