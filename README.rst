Markovpass
==========

A Markov chain based passphrase generator with entropy estimation.
``markovpass`` generates randomized passphrases based on a Markov chain along
with the total Shannon entropy of the nodes traversed. Long random sequences of
characters are difficult to remember. Shorter, or less random sequences are bad
passphrases. Long sequences of words are relatively `easy to remember
<https://xkcd.com/936/>`_ but take a long time to type.  ``markovpass``
generates human sounding phrases, which aim to strike a balance between ease of
memorization, length, and passphrases quality. The passphrases produced look
something like::

    qurken ret which bettle nurence
    
or::

    facupid trible taxed partice

Usage
-----

::

  Usage: markovpass [FILE] [options]

  Options:
      -n NUM              Number of passphrases to generate (default 1)
      -e MINENTROPY       Minimum entropy (default 60)
      -l LENGTH           NGram length (default 3)
      -w LENGTH           Minimum word length for corpus (default 5)
      -h, --help          Display this help and exit

``markovpass`` requires a corpus to work with. The corpus can either be
provided via the ``FILE`` argument or via STDIN. It can take pretty much any
text input and will strip the input of any non-alphabetic characters (and
discard words containing non-alphabetic characters, but keep words sandwiched
by non-alphabetic characters). The larger and more varied the corpus the
greater the entropy, but you'll hit diminishing returns fairly quickly. I
recommend using `Project Guttenberg <https://www.gutenberg.org/>`_; personally
I like a mix of H.P. Lovecraft and Jane Austen. The ``-w`` option can be used
to remove short words from the corpus which will increase the average length of
words in your passphrase, but not guarantee a minimum length (the minimum word
length will be the lesser of the ``-w`` and ``-l`` options). Obviously
increasing the minimum word length will lead to longer passphrases for the same
entropy.

Shannon Entropy and Guesswork
-----------------------------

Shannon entropy provides a good estimate of the lower bound of the average
guesswork required to guess a passphrases (to within an equivalent of a little
over 1.47 bits) [1]_, but average guesswork is not necessarily a reliable proxy
for difficulty in guessing a passphrase [2]_. Consider the following
passphrases generation method: I choose 500 characters and for each character
there is a 0.999 chance I choose 'a' and a 0.001 chance I choose 'b'. The
Shannon entropy for this process is about 5.7 bits, which based on [1]_ should
give an average number of guesses needed of at least 17.9. Yet an adversary who
knows my method will guess 'aaaaa...' and get my passphrases right on the first
guess 60.4% of the time. So you should treat Shannon entropy estimates with
caution.

That said, I suspect that for moderately long ``markovpass`` passphrases
using a representative corpus of language, Shannon entropy is probably a good
proxy for difficulty in guessing. The fundamental problem with average
guesswork is that the distribution of passphrase probabilities isn't
necessarily flat. If the distribution has a strong peak (or multiple peaks) and
a long tail of lower probability passphrases then average guesswork is going to
be a poor proxy for the strength of the passphrase generation method. In the
case of ``markovpass``, if trained on a reasonably representative corpus of
language, over a large enough series of decisions the probability distribution
of passphrases should look more or less gaussian (Some variant of the `Central
limit theorem <https://en.wikipedia.org/wiki/Central_limit_theorem>`_ for
Markov Chains should apply). While a Gaussian distribution isn't a flat
distribution, it's also a long ways from the pathological example above. The
Shannon entropy given is definitely an overestimate of the difficulty in
guessing, but probably not a terrible one.

.. [1] J. L. Massey, “Guessing and entropy,” in Proc. IEEE Int. Symp.
  Information Theory, 1994, p. 204.
.. [2] D. Malone and W.G. Sullivan, “Guesswork and Entropy,” IEEE Transactions
  on Information Theory, vol. 50, 525-526, March 2004.

Acknowledgements
----------------

Thanks to Gavin Wahl for the idea for this project, and also for, after I had
it working, questioning the whole idea, and forcing me to think about
guesswork, and Alex Bliskovsky for getting me to check out Rust.
