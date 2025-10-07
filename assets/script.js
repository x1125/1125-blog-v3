const markdownRenderer = window.markdownit().use(window.markdownitFootnote);

document.addEventListener("DOMContentLoaded", function () {
    window.addEventListener("hashchange", function () {
        DoRouting();
    }, false);

    DoRouting();
});

function DoRouting() {
    const requestParameter = String(window.location.hash).slice(1);

    if (requestParameter === '') {
        RecentlyUpdatedAction();
        return;
    }

    const requestParameters = requestParameter.split(':');
    const mdPath = 'posts/' + requestParameters[0] + '.md';
    RenderPostAction(mdPath);
}

function HttpGetRequest(url) {
    return new Promise(function (resolve, reject) {
        const xhr = new XMLHttpRequest();
        xhr.onload = () => {
            if (xhr.status >= 200 && xhr.status < 300) {
                resolve(xhr.responseText);
            } else {
                reject(xhr.status);
            }
        };
        xhr.onerror = () => {
            console.error(xhr);
            reject('unknown error');
        };
        xhr.open('GET', url, true);
        xhr.send();
    });
}

function RecentlyUpdatedAction() {
    SetMainContent('asdasd');
    ShowContentContainer('main');
}

function RenderPostAction(mdPath) {
    ShowContentContainer('loading');
    HttpGetRequest(mdPath).then((markdownData) => {
        SetMainContent(markdownRenderer.render(markdownData));
        ShowContentContainer('main');
    }).catch((error) => {
        if (error === 404) {
            SetMessageBox('warning', 'Page not found', 'The Page could not be found, please try again');
            ShowContentContainer('message');
            return;
        }
        SetMessageBox('danger', 'Unknown error', 'Please try reloading the page');
        ShowContentContainer('message');
    });
}

function ShowContentContainer(type) {
    const contentContainers = document.getElementsByClassName('content-container');
    Array.prototype.forEach.call(contentContainers, function(contentContainer) {
        contentContainer.classList.add('is-hidden');
    });
    document.getElementById(type + '-container').classList.remove('is-hidden');
}

function SetMainContent(content) {
    document.getElementById('main-container').innerHTML = content;
}

function SetMessageBox(type, title, body) {
    document.querySelector('#message-container').classList.remove(['is-dark', 'is-primary', 'is-link', 'is-info', 'is-success', 'is-warning', 'is-danger']);
    document.querySelector('#message-container').classList.add('is-' + type);
    document.querySelector('#message-container .message-header > p').innerHTML = title;
    document.querySelector('#message-container .message-body').innerHTML = body;
}