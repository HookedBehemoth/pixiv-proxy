function inject(element, after = true, parent = false) {
    var holder = element.parentNode
    console.log(holder)
    function insert(target, element) {
        if (after) {
            target.innerHTML += element
        } else {
            target.innerHTML = element + target.innerHTML
        }
    }
    let url = element.attributes.endpoint.value
    var request = new XMLHttpRequest()
    request.open('GET', url, true)
    request.onload = function() {
        let spinner = holder.getElementsByClassName('spinner')[0]
        spinner.remove()

        if (this.status == 200) {
            var target = holder
            if (parent) {
                target = holder.parentNode
                holder.remove()
            }
            insert(target, this.responseText)
        } else {
            insert(holder, `An error occured loading from ${url}`)
        }
    }
    request.send(null)
    element.remove()
    insert(holder, '<div class="spinner"></div>')
}
