function inject(element, after = true) {
    let holder = element.parentNode
    function insert(element) {
        if (after) {
            holder.innerHTML += element
        } else {
            holder.innerHTML = element + holder.innerHTML
        }
    }
    let url = element.attributes.endpoint.value
    var request = new XMLHttpRequest()
    request.open('GET', url, true)
    request.onload = function() {
        if (this.status == 200) {
            insert(this.responseText)
        } else {
            insert(`An error occured loading from ${url}`)
        }
        let spinner = holder.getElementsByClassName('spinner')[0]
        spinner.remove()
    }
    request.send(null)
    element.remove()
    insert('<div class="spinner"></div>')
}
