const list = document.createElement('ul');

const main = document.querySelector('main');
main.appendChild(list);

main.onclick = function() {
  const listContent = prompt('What content do you want the list item to have?');
  // On cancel, no list element should be created
  if (listContent == null) return;

  const listItem = document.createElement('li');
  listItem.textContent = listContent;
  list.appendChild(listItem);

  listItem.onclick = function(e) {
    e.stopPropagation();
    const listContent = prompt('Enter new content for your list item');
    this.textContent = listContent;
  }
}
