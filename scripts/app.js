'use strict';

const switcher = document.querySelector('.themeBtn');

switcher.addEventListener('click', function() {
    const className = document.body.className;
    switch (className) {
        case "light-theme":
            document.body.classList.toggle('light-theme');
            document.body.classList.toggle('dark-theme');
            this.textContent = "light"
            break;
        case "dark-theme":
            document.body.classList.toggle('light-theme');
            document.body.classList.toggle('dark-theme');
            this.textContent = "dark"
            break;
        default:
            document.body.classList.add('light-theme');
            document.body.classList.remove('dark-theme');
            this.textContent = "dark"
            break;
    }
});
