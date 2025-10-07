'use strict';

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

const Tags = {
    'in-progress': 'warning',
    'done': 'success'
};

const markdownRenderer = window.markdownit()
    .use(window.markdownitFootnote)
    .use(window.markdownitFootnoteBulma(Request))
    .use(window.markdownitTags(Tags));

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

    LoadMenu(() => DoRouting());
});

function LoadMenu(cb) {
    HttpGetRequest('post_index')
        .then((postIndexData) => {
            const postIndex = JSON.parse(postIndexData);
            RenderMenu(postIndex);
        })
        .finally(() => {
            if (cb) {
                cb();
            }
        });
}

function RenderMenu(postIndex) {
    const addSubNode = (prefix, pathArr) => {
        // starting with the first element
        const nodeName = pathArr.shift();

        // construct full path
        let path = nodeName;
        let hasPrefix = false;
        if (prefix.length > 0 && prefix[0].length > 0) {
            path = prefix.join('/') + '/' + path;
            hasPrefix = true;
        }

        // add only if not already exists
        if (document.querySelector(`a[href="#${path}"]`) === null) {
            let target = '#menuList';
            if (hasPrefix) {
                target += ' ' + 'ul[data-id="' + prefix.join('/') + '"]';

                if (document.querySelector(target) === null) {
                    document.querySelector('li[data-id="' + prefix.join('/') + '"]').innerHTML +=
                        '<ul data-id="' + prefix.join('/') + '"></ul>';
                }
            }
            document.querySelector(target).innerHTML += NewMenuNode(path, nodeName);
        }

        // element moved from pathArr to prefix; next iteration can start
        if (pathArr.length > 0 && pathArr[0].length > 0) {
            prefix.push(nodeName);
            addSubNode(prefix, pathArr);
        }
    };
    postIndex.forEach((post) => {
        addSubNode([], post.split('/'));
    });
}

function NewMenuNode(link, title) {
    return `<li data-id="${link}"><a href="#${link}">&raquo;&nbsp;${title}</a></li>`;
}

function DoRouting() {
    Request.parse(window.location.hash);

    let firstElement = '';
    if (Request.path.length > 0) {
        firstElement = Request.path[0];
    }

    SetActiveMenuItem(Request.path);

    if (firstElement === '') {
        window.location.hash = '#latest';
        return;
    }

    if (firstElement === 'latest') {
        RecentlyUpdatedAction();
        return;
    }

    RenderPostAction('posts/' + Request.buildPath() + '.md');
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
    Array.prototype.forEach.call(contentContainers,
        (contentContainer) => contentContainer.classList.add('is-hidden'));
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

function SetActiveMenuItem(routes) {
    let routeCombination = [];
    let routeSet = [];
    for (let i=0; i<routes.length; i++) {
        routeCombination.push(routes[i]);
        routeSet.push(routeCombination.join(pathDelimiter));
    }
    document.querySelectorAll('#menuList li a').forEach((a) =>
        a.classList.toggle('is-active', routeSet.indexOf(a.hash.substr(1)) !== -1)
    );
}

function ScrollToAnchor(id) {
    const targetElement = document.getElementById(id);
    if (!targetElement) {
        return;
    }
    (document.scrollingElement || document.documentElement).scrollTop = targetElement.offsetTop;
}