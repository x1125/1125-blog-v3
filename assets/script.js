const markdownRenderer = window.markdownit().use(window.markdownitFootnote);

const footnoteDelimiter = '^';

function filterFootnoteParameter(requestParameter) {
    return requestParameter.split(footnoteDelimiter)[0];
}

function getRequestParameterWithoutFootnote() {
    return filterFootnoteParameter(String(window.location.hash).split('#')[1]);
}

markdownRenderer.renderer.rules.footnote_ref = function(tokens, idx, options, env, slf) {
    const id = slf.rules.footnote_anchor_name(tokens, idx, options, env, slf);
    const caption = slf.rules.footnote_caption(tokens, idx, options, env, slf);
    let refid = id;

    if (tokens[idx].meta.subId > 0) {
        refid += ':' + tokens[idx].meta.subId;
    }

    return '<sup class="footnote-ref"><a href="#' + getRequestParameterWithoutFootnote() + footnoteDelimiter + id + '" id="' + getRequestParameterWithoutFootnote() + footnoteDelimiter + 'ref' + refid + '">' + caption + '</a></sup>';
};

markdownRenderer.renderer.rules.footnote_anchor = function(tokens, idx, options, env, slf) {
    let id = slf.rules.footnote_anchor_name(tokens, idx, options, env, slf);

    if (tokens[idx].meta.subId > 0) {
        id += ':' + tokens[idx].meta.subId;
    }

    /* ↩ with escape code to prevent display as Apple Emoji on iOS */
    return ' <a href="#' + getRequestParameterWithoutFootnote() + footnoteDelimiter + 'ref' + id + '" class="footnote-backref">\u21a9\uFE0E</a>';
};

markdownRenderer.renderer.rules.footnote_open = function(tokens, idx, options, env, slf) {
    var id = slf.rules.footnote_anchor_name(tokens, idx, options, env, slf);

    if (tokens[idx].meta.subId > 0) {
        id += ':' + tokens[idx].meta.subId;
    }

    return '<li id="' + getRequestParameterWithoutFootnote() + footnoteDelimiter + id + '" class="footnote-item">';
};

document.addEventListener("DOMContentLoaded", function () {
    window.addEventListener("hashchange", function (e) {
        if (filterFootnoteParameter(e.newURL) === filterFootnoteParameter(e.oldURL)) {
            // no change occurred
            // required for footnotes
            return;
        }
        DoRouting();
    }, false);

    DoRouting();
});

function DoRouting() {
    const requestParameter = getRequestParameterWithoutFootnote();

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