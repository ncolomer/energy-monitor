import logging
from concurrent.futures import ThreadPoolExecutor
from queue import Queue
from threading import Thread, Lock
from typing import Callable


class Dispatcher(Thread):
    """
    Class responsible for dispatching messages to subscribed handlers.
    """

    def __init__(self):
        super().__init__(name=self.__class__.__name__)
        self.logger = logging.getLogger(self.__class__.__name__)
        self.queue = Queue()
        self.subscription_lock = Lock()
        self.subscribers = []

    def subscribe(self, handler: Callable):
        with self.subscription_lock:
            subscriber_id = f'Subscriber{len(self.subscribers) + 1}'
            executor = ThreadPoolExecutor(thread_name_prefix=f'{self.name}-{subscriber_id}', max_workers=1)
            self.subscribers.append((executor, handler))
            self.logger.debug('Added subscriber %s', subscriber_id)

    def publish(self, message):
        self.queue.put(message)

    def run(self):
        while True:
            message = self.queue.get()
            for executor, handler in self.subscribers:
                executor.submit(handler, message)
            self.queue.task_done()


pubsub = Dispatcher()
pubsub.start()
