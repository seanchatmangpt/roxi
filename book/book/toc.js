// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">User Guide</li><li class="chapter-item expanded "><a href="guide/getting-started.html"><strong aria-hidden="true">1.</strong> Getting Started</a></li><li class="chapter-item expanded "><a href="guide/installation.html"><strong aria-hidden="true">2.</strong> Installation</a></li><li class="chapter-item expanded "><a href="guide/usage.html"><strong aria-hidden="true">3.</strong> Basic Usage</a></li><li class="chapter-item expanded "><a href="guide/queries.html"><strong aria-hidden="true">4.</strong> Running Queries</a></li><li class="chapter-item expanded affix "><li class="part-title">Reference Guide</li><li class="chapter-item expanded "><a href="reference/term-model.html"><strong aria-hidden="true">5.</strong> Term Model &amp; RDF 1.2</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="reference/iri-encoder.html"><strong aria-hidden="true">5.1.</strong> Iri &amp; Encoder</a></li><li class="chapter-item expanded "><a href="reference/literals.html"><strong aria-hidden="true">5.2.</strong> Literals &amp; Datatypes</a></li><li class="chapter-item expanded "><a href="reference/blank-nodes.html"><strong aria-hidden="true">5.3.</strong> Blank Nodes</a></li><li class="chapter-item expanded "><a href="reference/rdf-12-triples.html"><strong aria-hidden="true">5.4.</strong> RDF 1.2 Triple Terms</a></li></ol></li><li class="chapter-item expanded "><a href="reference/reasoning.html"><strong aria-hidden="true">6.</strong> Rule Reasoning Engine</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="reference/forward-chaining.html"><strong aria-hidden="true">6.1.</strong> Forward Chaining Reasoner</a></li><li class="chapter-item expanded "><a href="reference/backward-chaining.html"><strong aria-hidden="true">6.2.</strong> Backward Chainer</a></li><li class="chapter-item expanded "><a href="reference/cycle-safety.html"><strong aria-hidden="true">6.3.</strong> Cycle Safety &amp; Visited Guards</a></li><li class="chapter-item expanded "><a href="reference/csprite.html"><strong aria-hidden="true">6.4.</strong> CSprite Hierarchy Traversal</a></li></ol></li><li class="chapter-item expanded "><a href="reference/datalog.html"><strong aria-hidden="true">7.</strong> Advanced Datalog Engine</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="reference/datalog-safety.html"><strong aria-hidden="true">7.1.</strong> Rule Safety Checks</a></li><li class="chapter-item expanded "><a href="reference/datalog-negation.html"><strong aria-hidden="true">7.2.</strong> Stratified Negation</a></li><li class="chapter-item expanded "><a href="reference/datalog-aggregates.html"><strong aria-hidden="true">7.3.</strong> Aggregations &amp; Grouping</a></li></ol></li><li class="chapter-item expanded "><a href="reference/n3.html"><strong aria-hidden="true">8.</strong> Notation3 (N3) Specification</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="reference/n3-grammar.html"><strong aria-hidden="true">8.1.</strong> N3 Grammar &amp; Parser</a></li><li class="chapter-item expanded "><a href="reference/n3-formulae.html"><strong aria-hidden="true">8.2.</strong> Formulae &amp; Quoted Graphs</a></li><li class="chapter-item expanded "><a href="reference/n3-quantifiers.html"><strong aria-hidden="true">8.3.</strong> Quantifiers &amp; Scoping</a></li><li class="chapter-item expanded "><a href="reference/n3-lists.html"><strong aria-hidden="true">8.4.</strong> N3 Lists</a></li><li class="chapter-item expanded "><a href="reference/n3-builtins.html"><strong aria-hidden="true">8.5.</strong> Procedural Built-ins</a></li></ol></li><li class="chapter-item expanded "><a href="reference/validation.html"><strong aria-hidden="true">9.</strong> Shape Validation</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="reference/oxrdf-adapter.html"><strong aria-hidden="true">9.1.</strong> oxrdf Zero-Copy Adapter</a></li><li class="chapter-item expanded "><a href="reference/shacl.html"><strong aria-hidden="true">9.2.</strong> SHACL Core &amp; SPARQL Constraints</a></li><li class="chapter-item expanded "><a href="reference/shex.html"><strong aria-hidden="true">9.3.</strong> ShEx Expression &amp; ShapeMaps</a></li></ol></li><li class="chapter-item expanded "><a href="reference/sparql.html"><strong aria-hidden="true">10.</strong> SPARQL 1.1 Engine</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="reference/sparql-planning.html"><strong aria-hidden="true">10.1.</strong> Query Algebra &amp; Planning</a></li><li class="chapter-item expanded "><a href="reference/sparql-update.html"><strong aria-hidden="true">10.2.</strong> SPARQL 1.1 Update</a></li></ol></li><li class="chapter-item expanded "><li class="part-title">Continuous Integration &amp; Conformance</li><li class="chapter-item expanded "><a href="ci/workflows.html"><strong aria-hidden="true">11.</strong> CI Workflows</a></li><li class="chapter-item expanded "><a href="ci/conformance.html"><strong aria-hidden="true">12.</strong> Dialect Conformance Metrics</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded affix "><a href="misc/contributors.html">Contributors</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
