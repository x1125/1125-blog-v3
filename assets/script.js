const markdownRenderer = window.markdownit().use(window.markdownitFootnote);

const pathDelimiter = '/';
const sectionDelimiter = ':';
const footnoteDelimiter = '^';

const Request = {
    path: [],
    section: null,
    footnote: null,

    parse: function(requestParameter) {
        if (requestParameter.indexOf('#') === 0) {
            requestParameter = requestParameter.substr(1);
        }
        
        let tmp = requestParameter.split(footnoteDelimiter);
        this.footnote = tmp[1] || null;

        tmp = tmp[0].split(sectionDelimiter);
        this.section = tmp[1] || null;

        this.path = String(tmp[0]).split(pathDelimiter);
        
        return this;
    },
    
    buildPath: function() {
        return this.path.join(pathDelimiter);
    },
    buildSection: function() {
        return this.buildPath() + sectionDelimiter + this.section;
    },
    buildFootnote: function() {
        return this.buildPath() + footnoteDelimiter + this.footnote;
    }
};

markdownRenderer.renderer.rules.footnote_ref = function (tokens, idx, options, env, slf) {
    const id = slf.rules.footnote_anchor_name(tokens, idx, options, env, slf);
    const caption = slf.rules.footnote_caption(tokens, idx, options, env, slf);
    let refid = id;

    if (tokens[idx].meta.subId > 0) {
        refid += ':' + tokens[idx].meta.subId;
    }

    return '<sup class="footnote-ref"><a href="#' + Request.buildPath() + footnoteDelimiter + id +
        '" id="' + Request.buildPath() + footnoteDelimiter + 'ref' + refid + '">' + caption + '</a></sup>';
};

markdownRenderer.renderer.rules.footnote_anchor = function (tokens, idx, options, env, slf) {
    let id = slf.rules.footnote_anchor_name(tokens, idx, options, env, slf);

    if (tokens[idx].meta.subId > 0) {
        id += ':' + tokens[idx].meta.subId;
    }

    /* ↩ with escape code to prevent display as Apple Emoji on iOS */
    return ' <a href="#' + Request.buildPath() + footnoteDelimiter + 'ref' + id + '" class="footnote-backref">\u21a9\uFE0E</a>';
};

markdownRenderer.renderer.rules.footnote_open = function (tokens, idx, options, env, slf) {
    let id = slf.rules.footnote_anchor_name(tokens, idx, options, env, slf);

    if (tokens[idx].meta.subId > 0) {
        id += ':' + tokens[idx].meta.subId;
    }

    return '<li id="' + Request.buildPath() + footnoteDelimiter + id + '" class="footnote-item">';
};

markdownRenderer.renderer.rules.heading_open = function(tokens, idx, options, env, slf) {
    const title = tokens[idx+1].content;
    return '<div class="sectionLink"><' + tokens[idx].tag + ' id="' + Request.buildPath() + sectionDelimiter + title + '">';
};

markdownRenderer.renderer.rules.heading_close = function(tokens, idx, options, env, slf) {
    const title = tokens[idx-1].content;
    return '</' + tokens[idx].tag + '><a href="#' + Request.buildPath() + sectionDelimiter + title + '" class="is-hidden"><i class="fas fa-link"></i></a></div>';
};

document.addEventListener("DOMContentLoaded", function () {
    window.addEventListener("hashchange", function (e) {
        const newRequest = new Request.parse(e.newURL.split('#')[1]);
        if (newRequest.path.join(pathDelimiter) === Request.path.join(pathDelimiter)) {
            // no change occurred
            // required for footnotes
            return;
        }
        DoRouting();
    }, false);

    DoRouting();
});

function DoRouting() {
    Request.parse(window.location.hash);

    let firstElement = '';
    if (Request.path.length > 0) {
        firstElement = Request.path[0];
    }

    SetActiveMenuItem(firstElement);

    if (firstElement === '') {
        window.location.hash = '#latest';
        return;
    }

    if (firstElement === 'latest') {
        RecentlyUpdatedAction();
        return;
    }

    RenderPostAction('posts/' + firstElement + '.md');
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

        // TODO: scroll to section / footnote if available
        if (Request.section !== null) {
            ScrollToAnchor(Request.buildSection());
        } else if(Request.footnote !== null) {
            ScrollToAnchor(Request.buildFootnote());
        }

    }).catch((error) => {
        if (error === 404) {
            SetMessageBox('warning', 'Page not found', 'The Page could not be found, please try again');
            ShowContentContainer('message');
            return;
        }
        console.error(error);
        SetMessageBox('danger', 'Unknown error', 'Please try reloading the page');
        ShowContentContainer('message');
    });
}

function ShowContentContainer(type) {
    const contentContainers = document.getElementsByClassName('content-container');
    Array.prototype.forEach.call(contentContainers, function (contentContainer) {
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

function SetActiveMenuItem(route) {
    document.querySelectorAll('#menuList li a').forEach(function(a){
        if (a.hash === '#' + route) {
            a.classList.add('is-active');
        } else {
            a.classList.remove('is-active');
        }
    });
}

function ScrollToAnchor(id) {
    const targetElement = document.getElementById(id);
    if (!targetElement) {
        return;
    }
    (document.scrollingElement || document.documentElement).scrollTop = targetElement.offsetTop;
}