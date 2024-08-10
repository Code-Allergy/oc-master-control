
// handle 404 errors on the client (invalid link somewhere)
document.body.addEventListener('htmx:responseError', function(event) {
    if (event.detail.xhr.status === 404) {
        alert('This page does not exist. The feature may not be implemented yet.');
    }
});